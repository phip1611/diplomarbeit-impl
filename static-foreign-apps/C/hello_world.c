#include <stdio.h>
#include <unistd.h>
#include <fcntl.h>


int main(int argc, char *argv[], char * envp[]) {
	printf("hello world from linux written in C\n");

    // Opens a File, writes to it, and read from it afterwards.
    int fd = open("/tmp/foo.bar", O_CREAT | O_RDWR | O_TRUNC, 0777);
    printf("fd=%d\n", fd);
    int bytes_written = write(fd, "na moin :)", 10);
    printf("bytes written: %d\n", bytes_written);
    lseek(fd, 0, 0);
    char read_buf[11];
    read_buf[10] = 0; // Null terminated
    int bytes_read = read(fd, read_buf, 10);
    printf("bytes read: %d\n", bytes_read);
    printf("read: '%s'\n", read_buf);
}
