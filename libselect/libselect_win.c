// Includes
#include <stdint.h>
#include <Winsock2.h>
#include <fcntl.h>


// Constants
const uint8_t EVENT_NONE  = 0;
const uint8_t EVENT_READ  = 1 << 1;
const uint8_t EVENT_WRITE = 1 << 2;
const uint8_t EVENT_ERROR = 1 << 3;

const uint64_t INVALID_FD = ~0ULL;


int wait_for_event(uint64_t timeout_ms, uint64_t const* fds, uint8_t* events) {
	// Reset last error
	WSASetLastError(0);

	// Create select-sets
	fd_set read_set, write_set, error_set;
	FD_ZERO(&read_set );
	FD_ZERO(&write_set);
	FD_ZERO(&error_set);

	// Prepare sets
	SOCKET highest_fd = 0;
	for (size_t i = 0; fds[i] != INVALID_FD; i++) {
		// Capture FD and event
		SOCKET fd = (SOCKET)fds[i];
		uint8_t event = events[i];

		// Insert FD into sets
		if (event & EVENT_READ ) FD_SET(fd, &read_set );
		if (event & EVENT_WRITE) FD_SET(fd, &write_set);
		if (event & EVENT_ERROR) FD_SET(fd, &error_set);

		// Capture highest FD
		highest_fd = highest_fd < fd ? fd : highest_fd;
	}

	// Create timeval-struct
	struct timeval timeout;
	timeout.tv_sec = (long)timeout_ms / 1000;
	timeout.tv_usec = ((long)timeout_ms % 1000) * 1000;

	// Call select
	if (select((int)highest_fd + 1, &read_set, &write_set, &error_set, &timeout) == -1) return WSAGetLastError();

	// Check sets
	for (size_t i = 0; fds[i] != INVALID_FD; i++) {
		// Capture FD and set the event to `EVENT_NONE`
		SOCKET fd = (SOCKET)fds[i];
		events[i] = EVENT_NONE;

		// Check FDs for events
		if (FD_ISSET(fd, &read_set )) events[i] |= EVENT_READ;
		if (FD_ISSET(fd, &write_set)) events[i] |= EVENT_WRITE;
		if (FD_ISSET(fd, &error_set)) events[i] |= EVENT_ERROR;
	}
	return 0;
}

int set_blocking_mode(uint64_t fd, uint8_t blocking) {
	// Reset last error
	WSASetLastError(0);

	// Set blocking mode
	unsigned long mode = blocking ? 0 : 1;
	return (ioctlsocket((SOCKET)fd, FIONBIO, &mode) == 0) ? 0 : WSAGetLastError();
}
