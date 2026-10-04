#include<stdint.h>
#include<stdio.h>
#include<wayland-server-core.h>
#include<wayland-server.h>
#include<wayland-server-protocol.h>
#include<signal.h>
#include<stdlib.h>

// everything this server owns inito one file
// instead of making everything here and there 
// make once and edit the globals here
// as gloabals -> 1 but resources diff for all...
struct CompositorState{
    struct wl_display *display;
    struct wl_event_loop *loop;
    struct wl_event_source *timer;
    struct wl_event_source *signal;
    struct wl_global *compositor_global;
};

// static int 
// handle_timer(void *data){
//     // init the state to data...
//     struct CompositorState *state = data;
//     // incrememnt the tick
//     state -> tick_count++;
//     // proof of the tick increasing..
//     printf("timer tick #%d, timer is still running...\n", state->tick_count);
//     // as the tick count msg is one itime 
//     // we will call upon it every 2 sec...
//     wl_event_source_timer_update(state->timer, 2000);
//     // returning 0 as it means success!
//     return 0;
// }


// this function runs when an client asks for wl_compositor
// when an app says give me a surface to draw pixels on , this has to run...
static void
compositor_create_surface(struct wl_client *client, struct wl_resource *resource, uint32_t id){
    printf("CLIENT asked to create an object with OBJECT ID : %u! \n", id);
}

// when asked to create region this function runs...
static void
compositor_create_region(struct wl_client *client, struct wl_resource *resource, uint32_t id){
    printf("CLIENT asked to create a region with the OBJECT ID : %u! \n", id);
}

// implementation table 
// maps the protocol requests (opcode) tpo C functions...
// OPCODE 0 = create_surface
// OPCODE 1 = create_region

static const struct wl_compositor_interface compositor_implementation = {
    .create_surface = compositor_create_surface,
    .create_region = compositor_create_region,
};


// after creating the advertisement the client can now see that the conection can be formed..
// wl_resource is created so the client can bind to it...
// client = client requesting the bind
// data = user data passed during wl_global_create
// version = the protocol version the client requested
// id = the client-allocated OBJECT ID for this bound interface...
static void
compositor_bind(struct wl_client *client, void *data, uint32_t version, uint32_t id){
    printf("REGISTRY BIND : Client requested to bind 'wl_compositor'\n the requested version is %u and the OBJECT ID is %u \n", version, id);

    // allocating a server side resource
    // tracking this client's conenction to wl_compositor...
    struct wl_resource *resource = wl_resource_create(client, &wl_compositor_interface, version, id);

    if(!resource){
        wl_client_post_no_memory(client);
        return;
    }
    // setting the implementation table so that libwayalnd 
    // knows what C functions to call when messages arrive...
    wl_resource_set_implementation(resource, &compositor_implementation, data, NULL);
}


static int
signal_callback(int signal_number, void *data){
    // init the display
    struct CompositorState *state = data;
    // priting the proof message 
    printf("\n caught the sigint termination... exiting gracefully...");
    // closing the display so that 
    // the wayland lock is removed
    // and other things can work in it...
    wl_display_terminate(state->display);
    // 0 for the success number 
    // anything other then 0 is error...
    return 0;
}

int main(int argc, char *argv[]){
    // init the compositor state struct
    struct CompositorState state = {0};

    // creating the display
    state.display = wl_display_create();
    if(!state.display){
        fprintf(stderr, "FATAL ERROR : Could not create a Wayland display..");
        return EXIT_FAILURE;
    }

    // creating the event loop to do various things
    // as this has in built fucntions like
    // signal_callback and timer...
    state.loop = wl_display_get_event_loop(state.display);

    // creating the wayland socket...
    const char *socket = wl_display_add_socket_auto(state.display);
    if(!socket){
        fprintf(stderr, "FATAL : Unable to add socket to the Wayland display..]");
        wl_display_destroy(state.display);
        return EXIT_FAILURE;
    }

    printf("Server running on socket %s \n", socket);


    // registering the wl_compositor global in the server registry...
    // wl_create_global takes
    // display = what display instance to take...
    // &wl_compositor_interface = the protocol interface descriptor
    // version(4) = the max protocol version our compositor supports...
    // compositor bind = function pointer called whenever a client binds this global registry... 
    // resitering the global 1...
    state.compositor_global = wl_global_create(
        state.display,
        &wl_compositor_interface,
        4,
        &state,
        compositor_bind
    );

    if(!state.compositor_global){
        fprintf(stderr, "FATAL : Failed to register wl_compositor_global...\n");
        wl_display_destroy(state.display);
        return EXIT_FAILURE;
    }

    printf("SUCCESS Advertised wl_global_registry (v4) to registry\n"); 

    // to activate shared memory on the server
    // we need to set advertise the wl_shm global
    // so that the compositors/clients knows
    // that the global can be connected to...
    if(wl_display_init_shm(state.display)<0){
        fprintf(stderr, "FATAL Failed to init wl_shm...\n");
        wl_global_destroy(state.compositor_global);
        wl_display_destroy(state.display);
        return EXIT_FAILURE;
    }
    printf("Initialized the wl_shm successfully...\n");

    // adding the signal...
    state.signal = wl_event_loop_add_signal(state.loop, SIGINT, signal_callback, &state);

    // adding the timer
    // state.timer = wl_event_loop_add_timer(state.loop, handle_timer, &state);
    // wl_event_source_timer_update(state.timer, 2000);

    printf("Entering event loop\n");

    // running the display...
    wl_display_run(state.display);

    printf("Cleaning up resources...\n");
    // removing the added source timers 
    // and also destroying the display...
    // if(state.timer){
    //     wl_event_source_remove(state.timer);
    // }
    if(state.compositor_global){
        wl_global_destroy(state.compositor_global);
    }
    if(state.signal){
        wl_event_source_remove(state.signal);
    }
    wl_display_destroy(state.display);

    printf("Exited cleanly\n");
    return EXIT_SUCCESS;
}
