#include <stdio.h>
#include <wayland-server.h>

int main(int argc, char *argv[]){
    struct wl_display *display = wl_display_create();
    if(!display){
        fprintf(stderr, "The wayland display cannot be created...");
        return 1;
    }

    const char *socket = wl_display_add_socket_auto(display);
    if(!socket){
        fprintf(stderr, "Unable to add socket to wayland display...");
        return 1;
    }
    fprintf(stderr, "Running wayland display on %s\n", socket);
    wl_display_run(display);

    wl_display_destroy(display);
    return 0;
}
