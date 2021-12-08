#!/usr/bin/python3
from utils import SWHKD_UTILS
import grp
import pwd
import getpass
import libevdev
import sys

class SWHKD:
    """ 
    Main Class.
    """
    def __init__(self):
        self.utils = SWHKD_UTILS()
        self.user = getpass.getuser()

    def run_swhkd(self):

        groups = [g.gr_name for g in grp.getgrall() if self.user in g.gr_mem]
        gid = pwd.getpwnam(self.user).pw_gid
        groups.append(grp.getgrgid(gid).gr_name)
        for group in groups:
            if group.lower() == "input":
                self.utils.log_info("User is in input group, proceeding.")
                break;

        fd = open('/dev/input/event7','rb')
        device = libevdev.Device(fd)
        if not device.has(libevdev.EV_KEY.BTN_LEFT):
             print('This does not look like a mouse device')
             sys.exit(0)

        while True:
            for event in device.events():
                if not event.matches(libevdev.EV_KEY):
                    continue
                if event.matches(libevdev.EV_KEY.BTN_LEFT):
                    print('Left button event')
                elif event.matches(libevdev.EV_KEY.BTN_RIGHT):
                    print('Right button event')

SWHKD().run_swhkd()
