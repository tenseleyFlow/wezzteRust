Name:           wezztershier-gui
Version:        0.1.0
Release:        1%{?dist}
Summary:        Web GUI for Wezztershier WezTerm configuration tool

License:        MIT
URL:            https://github.com/tenseleyFlow/wezzteRust
Source0:        %{name}-%{version}.tar.gz

Requires:       wezztershier = 0.1.0
Requires:       python3

%description
Web-based GUI server for Wezztershier providing a beautiful visual interface
for configuring WezTerm with live preview and interactive widgets. Access via
web browser at http://localhost:8080 after running wezztershier-gui.
Includes built-in web server and responsive HTML interface.

Requires the base wezztershier package for CLI functionality.

%prep
%setup -q

%build
# No build needed - just shell script

%install
# Install GUI launcher
install -Dm755 wezztershier-gui %{buildroot}%{_bindir}/wezztershier-gui

%files
%{_bindir}/wezztershier-gui

%changelog
* Wed Jan 15 2025 espadonne (mfw) <espadonne@outlook.com> - 0.1.0-1
- Initial GUI package release
- Web-based interface with embedded HTML
- Requires base wezztershier package