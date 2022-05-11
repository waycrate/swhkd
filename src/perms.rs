use nix::unistd::{Gid, ResGid, ResUid, Uid, User};
use std::process::exit;

pub fn drop_privileges(user_uid: u32) {
    let user_uid = Uid::from_raw(user_uid);
    let user = User::from_uid(user_uid).unwrap().unwrap();

    set_initgroups(&user, user_uid.as_raw());
    set_egid(user_uid.as_raw());
    set_euid(user_uid.as_raw());
}

pub fn raise_privileges(resgid: ResGid, resuid: ResUid) {
    let root_user = User::from_uid(resuid.real).unwrap().unwrap();
    let resgid = resgid.effective.as_raw();
    let resuid = resuid.effective.as_raw();

    set_egid(resgid);
    set_euid(resuid);
    set_initgroups(&root_user, resgid);
}

pub fn get_resuid() -> nix::unistd::ResUid {
    match nix::unistd::getresuid() {
        Ok(resuid) => resuid,
        Err(e) => {
            log::error!("Failed to get RESUID: {:#?}", e);
            exit(1);
        }
    }
}

pub fn get_resgid() -> nix::unistd::ResGid {
    match nix::unistd::getresgid() {
        Ok(resgid) => resgid,
        Err(e) => {
            log::error!("Failed to get RESGID: {:#?}", e);
            exit(1);
        }
    }
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

fn set_resuid(ruid: u32, euid: u32, suid: u32) {
    let ruid = Uid::from_raw(ruid);
    let euid = Uid::from_raw(euid);
    let suid = Uid::from_raw(suid);

    println!("setresuid: {} {} {}", ruid, euid, suid);
    match nix::unistd::setresuid(ruid, euid, suid) {
        Ok(_) => log::debug!("Setting RESUID..."),
        Err(e) => {
            log::error!("Failed to set RESUID: {:#?}", e);
            exit(1);
        }
    }
}

fn set_resgid(rgid: u32, egid: u32, sgid: u32) {
    let rgid = Uid::from_raw(rgid);
    let egid = Uid::from_raw(egid);
    let sgid = Uid::from_raw(sgid);

    println!("setresgid: {} {} {}", rgid, egid, sgid);
    match nix::unistd::setresuid(rgid, egid, sgid) {
        Ok(_) => log::debug!("Setting RESUID..."),
        Err(e) => {
            log::error!("Failed to set RESGID: {:#?}", e);
            exit(1);
        }
    }
}
