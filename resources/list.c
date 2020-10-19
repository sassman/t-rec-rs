#include <string.h>

#include <xcb/xcb.h>
#include <xcb/xcb_atom.h>

// https://xcb.freedesktop.org/windowcontextandmanipulation/
// gcc list.c -o lst `pkg-config --cflags --libs xcb-atom`
// gcc list.c -o lst -lxcb
int
main ()
{
    /* open the connection to the X server */
    xcb_connection_t *connection = xcb_connect (NULL, NULL);


    /* get the first screen */
    xcb_screen_t *screen = xcb_setup_roots_iterator (xcb_get_setup (connection)).data;


    /* create the window */

    xcb_window_t window = xcb_generate_id (connection);
    xcb_create_window (connection,
                       0,                             /* depth               */
                       window,
                       screen->root,                  /* parent window       */
                       0, 0,                          /* x, y                */
                       250, 150,                      /* width, height       */
                       10,                            /* border_width        */
                       XCB_WINDOW_CLASS_INPUT_OUTPUT, /* class               */
                       screen->root_visual,           /* visual              */
                       0, NULL );                     /* masks, not used     */


    /* set the title of the window */

    char *title = "Hello World !";
    xcb_change_property (connection,
                         XCB_PROP_MODE_REPLACE,
                         window,
                         XCB_ATOM_WM_NAME,
                         XCB_ATOM_STRING,
                         8,
                         strlen (title),
                         title );


    /* set the title of the window icon */

    char *iconTitle = "Hello World ! (iconified)";
    xcb_change_property (connection,
                         XCB_PROP_MODE_REPLACE,
                         window,
                         XCB_ATOM_WM_ICON_NAME,
                         XCB_ATOM_STRING,
                         8,
                         strlen(iconTitle),
                         iconTitle);


    /* map the window on the screen */

    xcb_map_window (connection, window);
    xcb_flush (connection);


    /* event loop (in this case, no events to handle) */
    while (1) {}

    return 0;
}