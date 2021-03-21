#!/bin/bash

# This is just a little script that can be downloaded from the internet to
# install rustup. It just does platform detection, downloads the installer
# and runs it.

# This install script is intended to download and install the latest available
# release of Wasmer.
# It attempts to identify the current platform and an error will be thrown if
# the platform is not supported.
#
# Environment variables:
# - WASMER_DIR (optional): defaults to $HOME/.wasmer
#
# You can install using this script:
# $ curl https://raw.githubusercontent.com/wasmerio/wasmer-install/master/install.sh | sh

# https://raw.githubusercontent.com/wasmerio/wasmer-install/master/install.sh
# https://sh.rustup.rs/


set -euo pipefail
printf "\n"


reset="\033[0m"
red="\033[31m"
green="\033[32m"
yellow="\033[33m"
white="\033[37m"
bold="\e[1m"
dim="\e[2m"


RELEASES_URL="https://github.com/doctavious/doctavious-cli/releases"

DOCTAVIOUS_VERBOSE="verbose"
if [ -z "$DOCTAVIOUS_INSTALL_LOG" ]; then
  DOCTAVIOUS_INSTALL_LOG="$DOCTAVIOUS_VERBOSE"
fi


need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}

# Run a command that should never fail. 
# If the command fails execution will immediately terminate with an 
# error showing the failing command.
ensure() {
    if ! "$@"; then err "command failed: $*"; fi
}

# This wraps curl or wget. Try curl first, if not installed, use wget instead.
downloader() {
    local _dld
    local _ciphersuites
    if check_cmd curl; then
        _dld=curl
    elif check_cmd wget; then
        _dld=wget
    else
        _dld='curl or wget' # to be used in error message of need_cmd
    fi

    if [ "$1" = --check ]; then
        need_cmd "$_dld"
    elif [ "$_dld" = curl ]; then
        get_ciphersuites_for_curl
        _ciphersuites="$RETVAL"
        if [ -n "$_ciphersuites" ]; then
            curl --proto '=https' --tlsv1.2 --ciphers "$_ciphersuites" --silent --show-error --fail --location "$1" --output "$2"
        else
            echo "Warning: Not enforcing strong cipher suites for TLS, this is potentially less secure"
            if ! check_help_for "$3" curl --proto --tlsv1.2; then
                echo "Warning: Not enforcing TLS v1.2, this is potentially less secure"
                curl --silent --show-error --fail --location "$1" --output "$2"
            else
                curl --proto '=https' --tlsv1.2 --silent --show-error --fail --location "$1" --output "$2"
            fi
        fi
    elif [ "$_dld" = wget ]; then
        get_ciphersuites_for_wget
        _ciphersuites="$RETVAL"
        if [ -n "$_ciphersuites" ]; then
            wget --https-only --secure-protocol=TLSv1_2 --ciphers "$_ciphersuites" "$1" -O "$2"
        else
            echo "Warning: Not enforcing strong cipher suites for TLS, this is potentially less secure"
            if ! check_help_for "$3" wget --https-only --secure-protocol; then
                echo "Warning: Not enforcing TLS v1.2, this is potentially less secure"
                wget "$1" -O "$2"
            else
                wget --https-only --secure-protocol=TLSv1_2 "$1" -O "$2"
            fi
        fi
    else
        err "Unknown downloader"   # should not reach here
    fi
}

install() {
    downloader --check

    # identify platform based on uname output
    get_architecture || return 1
    local _arch="$RETVAL"
    assert_nz "$_arch" "arch"

    local _url="${RUSTUP_UPDATE_ROOT}/dist/${_arch}/rustup-init${_ext}"

    local _dir
    _dir="$(mktemp -d 2>/dev/null || ensure mktemp -d -t doctavious-cli)"
    local _file="${_dir}/rustup-init${_ext}"


    # assemble expected release artifact name
    # BINARY="doctavious-${OS}-${ARCH}.tar.gz"
    # if [ $# -eq 0 ]; then
    #     # The version was not provided, assume latest
    #     wasmer_download_json LATEST_RELEASE "$RELEASES_URL/latest"
    #     WASMER_RELEASE_TAG=$(echo "${LATEST_RELEASE}" | tr -s '\n' ' ' | sed 's/.*"tag_name":"//' | sed 's/".*//')
    #     printf "Latest release: ${WASMER_RELEASE_TAG}\n"
    # else
    #     WASMER_RELEASE_TAG="${1}"
    #     printf "Installing provided version: ${WASMER_RELEASE_TAG}\n"
    # fi

    ensure mkdir -p "$_dir"
    ensure downloader "$_url" "$_file" "$_arch"
    ensure chmod u+x "$_file"
    if [ ! -x "$_file" ]; then
        printf '%s\n' "Cannot execute $_file (likely because of mounting /tmp as noexec)." 1>&2
        printf '%s\n' "Please copy the file to a location where you can execute binaries and run ./rustup-init${_ext}." 1>&2
        exit 1
    fi

}


install "$@" || exit 1