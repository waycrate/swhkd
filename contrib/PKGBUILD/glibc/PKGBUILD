# Maintainer: Aakash Sharma <aakashsensharma@gmail.com>
pkgname='swhkd-git'
_pkgname="swhkd"
pkgver=1.1.7.480.ge4a1a89
pkgrel=1
arch=('x86_64')
url="https://github.com/waycrate/swhkd"
pkgdesc="A display server independent hotkey daemon inspired by sxhkd."
license=('BSD')
depends=('polkit')
makedepends=('rustup' 'make' 'git')
conflicts=('swhkd-musl-git')
source=("$_pkgname::git+https://github.com/waycrate/$_pkgname")
sha256sums=('SKIP')

build(){
	cd "$_pkgname"
	make setup
	make glibc
}

package() {
	cd "$_pkgname"
	install -Dm 755 ./bin/swhkd "$pkgdir/usr/bin/swhkd"
	install -Dm 755 ./bin/swhks "$pkgdir/usr/bin/swhks"
	install -Dm 644 ./com.github.swhkd.pkexec.policy "$pkgdir/usr/share/polkit-1/actions/com.github.swhkd.pkexec.policy"
	install -dm700 "$pkgdir/etc/swhkd/runtime"
	chown root:root "$pkgdir/etc/swhkd/runtime"
	chmod 700 "$pkgdir/etc/swhkd/runtime"
	chmod 644 "$pkgdir/usr/share/polkit-1/actions/com.github.swhkd.pkexec.policy"
	chown root:root "$pkgdir/etc/swhkd/runtime"
}

pkgver() {
	cd $_pkgname
	echo "$(grep '^version =' Cargo.toml|head -n1|cut -d\" -f2|cut -d\- -f1).$(git rev-list --count HEAD).g$(git rev-parse --short HEAD)"
}
