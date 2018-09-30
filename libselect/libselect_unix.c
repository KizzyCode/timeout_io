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


// Constants
const uint8_t EVENT_READ   = 1 << 1;
const uint8_t EVENT_WRITE  = 1 << 2;
const uint8_t EVENT_ERROR  = 1 << 3;
const uint8_t SYSCALL_ERROR = 1 << 7;


/// Select-wrapper
///
/// Note: We chose `uint64_t` as descriptor-type because it is able to hold normal
/// UNIX-descriptors (positive `int`s) as well as Windows-`SOCKET`s (`uint64_t`s)
uint8_t wait_for_event(uint64_t descriptor, uint8_t event, uint64_t timeout_ms) {
	int sock_native = (int)descriptor;
	
	// Prepare result and create select-sets
	uint8_t result = 0;
	fd_set read_set, write_set, error_set;
	FD_ZERO(&read_set);
	FD_ZERO(&write_set);
	FD_ZERO(&error_set);
	
	// Initialize sets
	if ((event & EVENT_READ ) != 0) FD_SET(sock_native, &read_set);
	if ((event & EVENT_WRITE) != 0) FD_SET(sock_native, &write_set);
	if ((event & EVENT_ERROR) != 0) FD_SET(sock_native, &error_set);
	
	// Create timeval-struct and call select
	struct timeval timeout = { timeout_ms / 1000, (timeout_ms % 1000) * 1000 };
	if (select(sock_native + 1, &read_set, &write_set, &error_set, &timeout) == -1) result |= SYSCALL_ERROR;
	
	// Check sets
	if (FD_ISSET(sock_native, &read_set)) result |= EVENT_READ;
	if (FD_ISSET(sock_native, &write_set)) result |= EVENT_WRITE;
	if (FD_ISSET(sock_native, &error_set)) result |= EVENT_ERROR;
	
	return result;
}

int get_errno() {
	return errno;
}