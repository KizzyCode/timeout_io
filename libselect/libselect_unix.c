//
//  libselect.c
//  libselect
//
//  Created by Keziah Biermann on 20.08.17.
//
//  All rights reserved.
//  Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:
//
//  Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.
//  Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.
//  THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.


// Includes
#include <stdint.h>
#include <sys/select.h>
#include <sys/socket.h>
#include <errno.h>
#include <string.h>
#include <unistd.h>
#include <fcntl.h>


// Constants
const uint8_t EVENT_NONE  = 0;
const uint8_t EVENT_READ  = 1 << 1;
const uint8_t EVENT_WRITE = 1 << 2;
const uint8_t EVENT_ERROR = 1 << 3;

const uint64_t INVALID_FD = ~0;


uint8_t wait_for_event(uint64_t timeout_ms, uint64_t const* fds, uint8_t* events) {
	// Reset errno
	errno = 0;

	// Create select-sets
	fd_set read_set, write_set, error_set;
	FD_ZERO(&read_set );
	FD_ZERO(&write_set);
	FD_ZERO(&error_set);

	// Prepare sets
	int highest_fd = 0;
	for (size_t i = 0; fds[i] != INVALID_FD; i++) {
		// Capture FD and event
		int fd = (int)fds[i];
		uint8_t event = events[i];

		// Insert FD into sets
		if (event & EVENT_READ ) FD_SET(fd, &read_set );
		if (event & EVENT_WRITE) FD_SET(fd, &write_set);
		if (event & EVENT_ERROR) FD_SET(fd, &error_set);

		// Capture highest FD
		highest_fd = highest_fd < fd ? fd : highest_fd;
	}

	// Create timeval-struct and call select
	struct timeval timeout = { timeout_ms / 1000, (timeout_ms % 1000) * 1000 };
	if (select(highest_fd + 1, &read_set, &write_set, &error_set, &timeout) == -1) return errno;

	// Check sets
	for (size_t i = 0; fds[i] != INVALID_FD; i++) {
		// Capture FD and set the event to `EVENT_NONE`
		int fd = (int)fds[i];
		events[i] = EVENT_NONE;

		// Check FDs for events
		if (FD_ISSET(fd, &read_set )) events[i] |= EVENT_READ;
		if (FD_ISSET(fd, &write_set)) events[i] |= EVENT_WRITE;
		if (FD_ISSET(fd, &error_set)) events[i] |= EVENT_ERROR;
	}
	return 0;
}

int set_blocking_mode(uint64_t fd, uint8_t blocking) {
	// Reset errno
	errno = 0;

	// Get current flags
	int flags = fcntl((int)fd, F_GETFL, 0);
	if (flags == -1) return errno;

	// Add new flag
	flags = blocking ? (flags & ~O_NONBLOCK) : (flags | O_NONBLOCK);
	return (fcntl((int)fd, F_SETFL, flags) == -1) ? 0 : errno;
}
