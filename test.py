#!/usr/bin/env python3

import sys
import libevdev


def print_capabilities(l):
    v = l.driver_version
    print("Input driver version is {}.{}.{}".format(v >> 16, (v >> 8) & 0xff, v & 0xff))
    id = l.id
    print("Input device ID: bus {:#x} vendor {:#x} product {:#x} version {:#x}".format(
        id["bustype"],
        id["vendor"],
        id["product"],
        id["version"],
    ))
    print("Input device name: {}".format(l.name))
    print("Supported events:")

    for t, cs in l.evbits.items():
        print("  Event type {} ({})".format(t.value, t.name))

        for c in cs:
            if t in [libevdev.EV_LED, libevdev.EV_SND, libevdev.EV_SW]:
                v = l.value[c]
                print("    Event code {} ({}) state {}".format(c.value, c.name, v))
            else:
                print("    Event code {} ({})".format(c.value, c.name))

            if t == libevdev.EV_ABS:
                a = l.absinfo[c]
                print("       {:10s} {:6d}".format('Value', a.value))
                print("       {:10s} {:6d}".format('Minimum', a.minimum))
                print("       {:10s} {:6d}".format('Maximum', a.maximum))
                print("       {:10s} {:6d}".format('Fuzz', a.fuzz))
                print("       {:10s} {:6d}".format('Flat', a.flat))
                print("       {:10s} {:6d}".format('Resolution', a.resolution))

    print("Properties:")
    for p in l.properties:
        print("  Property type {} ({})".format(p.value, p.name))


def print_event(e):
    print("Event: time {}.{:06d}, ".format(e.sec, e.usec), end='')
    if e.matches(libevdev.EV_SYN):
        if e.matches(libevdev.EV_SYN.SYN_MT_REPORT):
            print("++++++++++++++ {} ++++++++++++".format(e.code.name))
        elif e.matches(libevdev.EV_SYN.SYN_DROPPED):
            print(">>>>>>>>>>>>>> {} >>>>>>>>>>>>".format(e.code.name))
        else:
            print("-------------- {} ------------".format(e.code.name))
    else:
        print("type {:02x} {} code {:03x} {:20s} value {:4d}".format(e.type.value, e.type.name, e.code.value, e.code.name, e.value))


def main(args):
    path = args[1]
    try:
        with open(path, "rb") as fd:
            dev = libevdev.Device(fd)
            print_capabilities(dev)
            print("################################\n"
                  "#      Waiting for events      #\n"
                  "################################")

            while True:
                try:
                    for e in dev.events():
                        print_event(e)
                except libevdev.EventsDroppedException:
                    for e in dev.sync():
                        print_event(e)

    except KeyboardInterrupt:
        pass
    except IOError as e:
        import errno
        if e.errno == errno.EACCES:
            print("Insufficient permissions to access {}".format(path))
        elif e.errno == errno.ENOENT:
            print("Device {} does not exist".format(path))
        else:
            raise e


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: {} /dev/input/eventX".format(sys.argv[0]))
        sys.exit(1)
    main(sys.argv)
