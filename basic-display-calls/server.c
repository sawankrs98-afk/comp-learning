#include<stdio.h>
#include<wayland-server.h>
#include<signal.h>
#include<stdlib.h>

// everything this server owns into one file
struct CompositorState{
    struct wl_display *display;
    struct wl_event_loop *loop;
    struct wl_event_source *timer;
    struct wl_event_source * signal;
    int tick_count;
};

static int 
handle_timer(void *data){
    // init the state to data...
    struct CompositorState *state = data;
    // incrememnt the tick
    state -> tick_count++;
    // proof of the tick increasing..
    printf("timer tick #%d, timer is still running...\n", state->tick_count);
    // as the tick count msg is one time 
    // we will call upon it every 2 sec...
    wl_event_source_timer_update(state->timer, 2000);
    // returning 0 as it means success!
    return 0;
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


    // adding the signal...
    state.signal = wl_event_loop_add_signal(state.loop, SIGINT, signal_callback, &state);

    // adding the timer
    state.timer = wl_event_loop_add_timer(state.loop, handle_timer, &state);
    wl_event_source_timer_update(state.timer, 2000);

    printf("Entering event loop");

    // running the display...
    wl_display_run(state.display);

    printf("Cleaning up resources...");
    // removing the added source timers 
    // and also destroying the display...
    if(state.timer){
        wl_event_source_remove(state.timer);
    }
    if(state.signal){
        wl_event_source_remove(state.signal);
    }
    wl_display_destroy(state.display);

    printf("Exited cleanly");
    return EXIT_SUCCESS;
}