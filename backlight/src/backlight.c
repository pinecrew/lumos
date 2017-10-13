#include <xcb/randr.h>
#include <xcb/xcb.h>
#include <xcb/xcb_util.h>
#include <xcb/xproto.h>

#include <ctype.h>
#include <limits.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static xcb_connection_t *conn;
static xcb_randr_output_t output;
static xcb_atom_t backlight;
static int32_t min;
static int32_t max;

enum {
    E_PROP_REPLY = -INT_MIN,
    E_PROP_FORMAT,
    E_VER_REPLY,
    E_VERSION,
    E_BACKLIGHT_REPLY,
    E_XCB_NONE,
    E_RESOURCE_REPLY,
    E_PROP_REPLY_RANGE,
    E_SUCCESS = 0
};

int32_t backlight_min() { return min; }
int32_t backlight_max() { return max; }

int32_t backlight_get() {
    xcb_generic_error_t *error;
    xcb_randr_get_output_property_reply_t *prop_reply = NULL;
    xcb_randr_get_output_property_cookie_t prop_cookie;
    int32_t value;

    if (backlight != XCB_ATOM_NONE) {
        prop_cookie = xcb_randr_get_output_property(conn, output, backlight, XCB_ATOM_NONE, 0, 4, 0, 0);
        prop_reply = xcb_randr_get_output_property_reply(conn, prop_cookie, &error);
        if (error != NULL || prop_reply == NULL) {
            return E_PROP_REPLY;
        }
    }

    if (prop_reply == NULL || prop_reply->type != XCB_ATOM_INTEGER || prop_reply->num_items != 1 ||
        prop_reply->format != 32) {
        value = E_PROP_FORMAT;
    } else {
        value = *((int32_t *)xcb_randr_get_output_property_data(prop_reply));
    }

    free(prop_reply);
    return value;
}

void backlight_set(int32_t value) {
    xcb_randr_change_output_property(conn, output, backlight, XCB_ATOM_INTEGER, 32, XCB_PROP_MODE_REPLACE, 1,
                                     (unsigned char *)&value);
    xcb_flush(conn);
}

int backlight_init() {
    xcb_generic_error_t *error;

    conn = xcb_connect(NULL, NULL);
    xcb_randr_query_version_cookie_t ver_cookie = xcb_randr_query_version(conn, 1, 2);
    xcb_randr_query_version_reply_t *ver_reply = xcb_randr_query_version_reply(conn, ver_cookie, &error);
    if (error != NULL || ver_reply == NULL) {
        return E_VER_REPLY;
    }
    if (ver_reply->major_version != 1 || ver_reply->minor_version < 2) {
        return E_VERSION;
    }
    free(ver_reply);

    xcb_intern_atom_cookie_t backlight_cookie = xcb_intern_atom(conn, 1, strlen("Backlight"), "Backlight");

    xcb_intern_atom_reply_t *backlight_reply = xcb_intern_atom_reply(conn, backlight_cookie, &error);
    if (error != NULL || backlight_reply == NULL) {
        return E_BACKLIGHT_REPLY;
    }

    backlight = backlight_reply->atom;
    free(backlight_reply);

    if (backlight == XCB_NONE) {
        return E_XCB_NONE;
    }

    xcb_screen_iterator_t iter = xcb_setup_roots_iterator(xcb_get_setup(conn));
    xcb_screen_t *screen = iter.data;
    xcb_window_t root = screen->root;
    xcb_randr_get_screen_resources_cookie_t resources_cookie = xcb_randr_get_screen_resources(conn, root);
    xcb_randr_get_screen_resources_reply_t *resources_reply =
        xcb_randr_get_screen_resources_reply(conn, resources_cookie, &error);
    if (error != NULL || resources_reply == NULL) {
        return E_RESOURCE_REPLY;
    }

    xcb_randr_output_t *outputs = xcb_randr_get_screen_resources_outputs(resources_reply);
    output = outputs[0];
    int32_t cur = backlight_get();
    if (cur != -1) {
        xcb_randr_query_output_property_cookie_t prop_cookie;
        xcb_randr_query_output_property_reply_t *prop_reply;

        prop_cookie = xcb_randr_query_output_property(conn, output, backlight);
        prop_reply = xcb_randr_query_output_property_reply(conn, prop_cookie, &error);
        if (error != NULL || prop_reply == NULL) {
            return -1;
        }

        if (prop_reply->range && xcb_randr_query_output_property_valid_values_length(prop_reply) == 2) {
            int32_t *values = xcb_randr_query_output_property_valid_values(prop_reply);
            min = values[0];
            max = values[1];
        } else {
            return E_PROP_REPLY_RANGE;
        }
    }
    return E_SUCCESS;
}