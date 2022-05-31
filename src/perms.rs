use nix::unistd::{Gid, Uid, User};
use std::process::exit;

pub fn drop_privileges(user_uid: u32) {
    let user_uid = Uid::from_raw(user_uid);
    let user = User::from_uid(user_uid).unwrap().unwrap();

    set_initgroups(&user, user_uid.as_raw());
    set_egid(user_uid.as_raw());
    set_euid(user_uid.as_raw());
}

pub fn raise_privileges() {
    let root_user = User::from_uid(Uid::from_raw(0)).unwrap().unwrap();

    set_egid(0);
    set_euid(0);
    set_initgroups(&root_user, 0);
}

fn set_initgroups(user: &nix::unistd::User, gid: u32) {
    let gid = Gid::from_raw(gid);
    match nix::unistd::initgroups(&user.gecos, gid) {
        Ok(_) => log::debug!("Setting initgroups..."),
        Err(e) => {
            log::error!("Failed to set init groups: {:#?}", e);
            exit(1);
        }
    }
}

fn set_egid(gid: u32) {
    let gid = Gid::from_raw(gid);
    match nix::unistd::setegid(gid) {
        Ok(_) => log::debug!("Setting EGID..."),
        Err(e) => {
            log::error!("Failed to set EGID: {:#?}", e);
            exit(1);
        }
    }
}

fn set_euid(uid: u32) {
    let uid = Uid::from_raw(uid);
    match nix::unistd::seteuid(uid) {
        Ok(_) => log::debug!("Setting EUID..."),
        Err(e) => {
            log::error!("Failed to set EUID: {:#?}", e);
            exit(1);
        }
    }
}
