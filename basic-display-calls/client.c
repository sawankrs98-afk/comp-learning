#include <stddef.h>
#define _GNU_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include <string.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/mman.h>
#include <wayland-client.h>

// client toolset 
// stores every registry we might encounter...
struct ClientState{
    struct wl_display *display;
    struct wl_registry *registry;
    struct wl_compositor *compositor;
    struct wl_surface *surface;
    struct wl_shm *shm;
};

// helper function : allocate anonymous RAM Files via memfd_create...
// memfd_create --> creates RAM backed file descriptor
// MFD_CLOEXEC --> ensures that the descriptor closes on exec()
// ftruncate --> sets the exact byte capacity if the RAM block....

static int
create_shm(size_t size){
    int fd = memfd_create("shm_pixel_buffer", MFD_CLOEXEC);
    if(fd<0){
        perror("memfd create failed...");
        return -1;
    }
    if(ftruncate(fd, size)<0){
        perror("ftruncate failed...");
        close(fd);
        return -1;
    }
    return fd;
}

// registry event 
// the billboard shows two methods 
// wl_registry
// wl_shm
// need to implement both!!...
static void 
registry_handle_global(void *data,
                       struct wl_registry *registry,
                       uint32_t name,
                       const char *interface,
                       uint32_t version){
    struct ClientState *state = (struct ClientState *)data;

    printf("REGISTRY EVENT Sever advertised interface : '%s' (NUMERIC ID : %u), Version : %u \n", interface, name, version);
    if(strcmp(interface, wl_compositor_interface.name)==0){
        printf("REGISTRY ACTION : Interface name wl_compositor... Binding now...!\n");

        state->compositor = wl_registry_bind(
            registry,
            name,
            &wl_compositor_interface,
            4
        );

        if(!state->compositor){
            fprintf(stderr, "ERROR Failed to bind to wl_compositor interface...\n");
        }
    }
    else if(strcmp(interface, wl_shm_interface.name) == 0){
        printf("REGISTRY name wl_shm binding now!\n");

        state -> shm = wl_registry_bind(registry, name, &wl_shm_interface, version);

        if(!state->shm){
            fprintf(stderr, "ERROR Failed to bind to wl_shm interface...\n");
        }
    }
}

static void
registry_handle_global_remove(void *data,
                              struct wl_registry *registry,
                              uint32_t name){
    printf("REGISTRY EVENT global numeric ID %u was removed by the compositor \n", name);
}

static const struct wl_registry_listener registry_listener = {
    .global = registry_handle_global,
    .global_remove = registry_handle_global_remove,
};

int 
main(int argc, char *argv[]) {
    struct ClientState state = {0};

    state.display = wl_display_connect(NULL);
    if(!state.display){
        fprintf(stderr, "Couldnt connect to the display\n");
        return EXIT_FAILURE;
    }
    
    printf("The wayland display successfully connected to the socket...\n");

    state.registry = wl_display_get_registry(state.display);
    if(!state.registry){
        fprintf(stderr, "FAILED to request wl_register...\n");
        wl_display_disconnect(state.display);
        return EXIT_FAILURE;
    }

    wl_registry_add_listener(state.registry, &registry_listener, &state);

    printf("Performing round trip to sync all the things before executing code \n");
    wl_display_roundtrip(state.display);

    if(state.compositor!=NULL&&state.shm!=NULL){
        // if the compositor is bound create a surface...
        printf("Compositor bound successfully...Creating a wl_surface\n");
        state.surface = wl_compositor_create_surface(state.compositor);
        // if the surface is not created 
        if(!state.surface){
            fprintf(stderr, "ERROR failed to give the wl_surface proxy...\n");
        } else {
            // calculate the dimension and stride...
            // 100*100 screen
            // 4 colour channel XRGB
            // 100*4 = 400 bytes per horizontal line..
            // total = 400*100 = 40000
            const int width = 100;
            const int height = 100;
            const int stride = width*4;
            const size_t size = stride*height;

            int fd = create_shm(size);
            if(fd>=0){
                // MAP RAM AND PAINT PIXELS...
                // Colouring to cyan here.. by getting the pointer to the RAM...
                // mmap() --> maps the file pages into client virtual adress space..
                uint32_t *pixels = mmap(NULL, size, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0);
                if(pixels!=MAP_FAILED){
                    for(int i=0;i<width*height;i++){
                        pixels[i]=0x0000FFFF;
                    }

                    // create pool and buffer....
                    // now that the ram pointer pixel has the colour
                    // hand the fd (ram key) --> to the server 
                    // done by wl_shm_create_pool...
                    // then slice out wl_buffer of 100*100...
                    // so it transfers fd over the socket using SHM rights...
                    struct wl_shm_pool *pool = wl_shm_create_pool(state.shm, fd, size);
                    struct wl_buffer *buffer = wl_shm_pool_create_buffer(pool, 
                                                                         0, 
                                                                         width,
                                                                         height,
                                                                         stride,
                                                                         WL_SHM_FORMAT_XRGB8888 // 32 BIT PIXEL FORMAT...
                    );

                    wl_shm_pool_destroy(pool);
                    close(fd);

                    // now attach the buffer and commit it...
                    wl_surface_attach(state.surface, buffer, 0, 0);
                    wl_surface_commit(state.surface);
                    
                    //flushes to the display immediately...
                    wl_display_flush(state.display);

                    printf("Request queued and flushed to server successfully...\n");
                    
                    // letting the server process the commit first...
                    wl_display_roundtrip(state.display);

                    // clean up memory mapping and buffer proxy...
                    munmap(pixels, size);
                    wl_buffer_destroy(buffer);
                }
            }

            // destroy bound interface proxies....
            
            wl_surface_destroy(state.surface);
            printf("SURFACE DESTROYED CLEANLY...\n");
        }
        
        wl_shm_destroy(state.shm);
    } else {
        printf("wl_compositor was not advertised by the compositor...\n");
    }

    printf("cleaning everything gracefully...\n");
    wl_registry_destroy(state.registry);
    wl_display_disconnect(state.display);

    printf("Disconnected cleanly...\n");
    return EXIT_SUCCESS;
}
