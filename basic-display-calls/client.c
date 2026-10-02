#include <stdint.h>
#include <stdio.h>
#include <wayland-client-core.h>
#include <wayland-client-protocol.h>
#include<string.h>
#include<stdlib.h>
#include <wayland-client.h>

// client toolset 
// stores every registry we might encounter...
struct ClientState{
    struct wl_display *display;
    struct wl_registry *registry;
    struct wl_compositor *compositor;
    struct wl_surface *surface;
};

// registry event 
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

    if(state.compositor!=NULL){
        printf("Compositor bound successfully...Creating a wl_surface\n");
        state.surface = wl_compositor_create_surface(state.compositor);

        if(!state.surface){
            fprintf(stderr, "ERROR failed to give the wl_surface proxy...\n");
        }else{
            wl_display_flush(state.display);
            printf("request queued and flushed to server successfully\n");

            wl_surface_destroy(state.surface);
            printf("Surface destroyed cleanly...\n");
        }

        wl_compositor_destroy(state.compositor);
    }else{
        printf("wl_compositor was not advertised by the compositor...\n");
    }

    printf("cleaning everything gracefully...\n");
    wl_registry_destroy(state.registry);
    wl_display_disconnect(state.display);

    printf("Disconnected cleanly...\n");
    return EXIT_SUCCESS;
}
