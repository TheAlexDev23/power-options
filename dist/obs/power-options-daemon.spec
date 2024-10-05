Name:      power-options-daemon
Version:   1.1.0
Release:   0
Summary:   The core daemon for Power Options, a blazingly fast power management solution.

License:   MIT
URL:       https://github.com/thealexdev23/power-options

Source0:   power-options-%{version}.tar.zst
Source1:   vendor.tar.zst

BuildRequires:   cargo

%if "%{?fedora}" == "rawhide"
BuildRequires:   llvm-libs
BuildRequires:   llvm-devel
%endif

Requires:        acpid
Requires:        pciutils
Requires:        usbutils
Requires:        yad

Suggests:        brightnessctl
Suggests:        net-tools
Suggests:        xorg-xrandr
Suggests:        xorg-xset
Suggests:        xautolock

%description
The core daemon for Power Options, a blazingly fast power management solution.

%prep
%setup -n power-options-%{version}
tar -xf %{_sourcedir}/vendor.tar.zst --directory .

%build
export VERGEN_GIT_DESCRIBE=v%{version}
cargo build --release --package power-daemon-mgr

%install
%{__install} -Dm755 target/release/power-daemon-mgr %{buildroot}%{_bindir}/power-daemon-mgr
%{buildroot}%{_bindir}/power-daemon-mgr -v generate-base-files --path %{buildroot} --program-path "%{_bindir}/power-daemon-mgr"

%post
if [ "$1" -eq 1 ]; then
    power-daemon-mgr setup
    systemctl daemon-reload
    systemctl restart acpid.service
    systemctl enable --now power-options.service
fi

%postun
systemctl daemon-reload

%files
%{_bindir}/power-daemon-mgr
/usr/lib/udev/rules.d/85-power-daemon.rules
/etc/acpi/events/power-options
/usr/share/dbus-1/system.d/power-daemon.conf
/usr/lib/systemd/system/power-options.service

%changelog
* Sat Sep 28 2024 Alexander Karpukhin <morskoyvolchonok@protonmail.com> - 1.1.0
- First version being packaged

