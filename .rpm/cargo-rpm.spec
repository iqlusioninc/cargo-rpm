%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: cargo-rpm
Summary: Build RPMs from Rust projects using Cargo
Version: @@VERSION@@
Release: 1
License: ASL 2.0
Group: Applications/System
Source0: %{name}-%{version}.tar.gz
URL: https://github.com/iqlusion-io/crates/
Requires: rpm-build >= 4

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

%description
%{summary}

%prep
%setup -q

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*

