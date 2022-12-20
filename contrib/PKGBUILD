# Maintainer: Aakash Sharma <aakashsensharma@gmail.com>
# Contributor: Sergey A. <murlakatamenka@disroot.org>
# Contributor: rv178 <idliyout@gmail.com>

_pkgname="swhkd"
pkgname="${_pkgname}-git"
pkgver=1.2.1.r17.g022466e
pkgrel=1
arch=("x86_64")
url="https://github.com/waycrate/swhkd"
pkgdesc="A display server independent hotkey daemon inspired by sxhkd."
license=("BSD")
depends=("polkit")
makedepends=("rustup" "make" "git" "scdoc")
conflicts=("swhkd-musl-git")
source=("${_pkgname}::git+${url}.git"
        "${_pkgname}-vim::git+${url}-vim.git")
sha256sums=("SKIP"
            "SKIP")

build(){
	cd "$_pkgname"
	make setup
	make
}

package() {
	cd "$_pkgname"
	make DESTDIR="$pkgdir/" install

    cd "${srcdir}/${_pkgname}-vim"
    for i in ftdetect ftplugin indent syntax; do
        install -Dm644 "$i/${_pkgname}.vim" \
            "${pkgdir}/usr/share/vim/vimfiles/$i/${_pkgname}.vim"
    done
}

pkgver() {
	cd $_pkgname
    git describe --long --tags --match'=[0-9]*' | sed 's/\([^-]*-g\)/r\1/;s/-/./g'
}
