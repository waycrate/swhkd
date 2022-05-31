# Maintainer: Aakash Sharma <aakashsensharma@gmail.com>
# Contributor: Sergey A. <murlakatamenka@disroot.org>

pkgname='swhkd-musl-git'
_pkgname="swhkd"
pkgver=1
pkgrel=1
arch=('x86_64')
url="https://github.com/waycrate/swhkd"
pkgdesc="A display server independent hotkey daemon inspired by sxhkd."
license=('BSD')
conflicts=('swhkd-glib-git')
depends=('polkit')
makedepends=('rustup' 'make' 'git')
source=("$_pkgname::git+https://github.com/waycrate/$_pkgname")
sha256sums=('SKIP')

build(){
	cd "$_pkgname"
	make setup
	make
}

package() {
	cd "$_pkgname"
	install -Dm 755 ./bin/swhkd "$pkgdir/usr/bin/swhkd"
	install -Dm 755 ./bin/swhks "$pkgdir/usr/bin/swhks"

	install -Dm 644 -o root ./com.github.swhkd.pkexec.policy -t "$pkgdir/usr/share/polkit-1/actions"
}

pkgver() {
	cd $_pkgname
	echo "$(grep '^version =' Cargo.toml|head -n1|cut -d\" -f2|cut -d\- -f1).$(git rev-list --count HEAD).g$(git rev-parse --short HEAD)"
}
