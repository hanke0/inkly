# shellcheck shell=bash
# Bootstrap Intel oneAPI / MKL for Cargo builds that use `candle-core` with feature `mkl`.
#
# The intel-mkl-src crate only downloads MKL 2020.1 when no system MKL is found; that
# bundle does not export hgemm_, which Candle requires for f16 matmul → link failure.
# Install `intel-oneapi-mkl-devel` (or set MKLROOT to a full oneAPI MKL tree) so
# intel-mkl-src links against a current MKL instead.

inkly_oneapi_mkl_env() {
	local _s
	case "$(uname -s)" in
	Linux)
		if [[ -n "${MKLROOT:-}" ]]; then
			return 0
		fi
		for _s in /opt/intel/oneapi/setvars.sh "${HOME}/intel/oneapi/setvars.sh"; do
			if [[ -f "${_s}" ]]; then
				# shellcheck disable=SC1090
				source "${_s}" >/dev/null 2>&1 || source "${_s}"
				break
			fi
		done
		;;
	esac
}

inkly_mkl_preflight_or_die() {
	if [[ -n "${MKLROOT:-}" ]] && [[ -e "${MKLROOT}/lib/libmkl_intel_lp64.a" || -e "${MKLROOT}/lib/intel64/libmkl_intel_lp64.a" ]]; then
		return 0
	fi
	# oneAPI layout: .../mkl/<version>/lib/intel64/
	if find /opt/intel/oneapi/mkl -name libmkl_intel_lp64.a -print -quit 2>/dev/null | grep -q .; then
		return 0
	fi
	cat >&2 <<'EOF'
inkly: Intel MKL not found (or MKLROOT not set).

Candle’s `mkl` feature needs a system MKL that provides hgemm_ (half-precision GEMM).
The copy intel-mkl-src downloads automatically (MKL 2020.1) is too old and will fail to link.

Fix on Debian/Ubuntu or WSL:
  • Add Intel’s oneAPI apt repo, then:
      sudo apt install intel-oneapi-mkl-devel libomp-dev
  • Run this script again, or:
      source /opt/intel/oneapi/setvars.sh
      export MKLROOT  # setvars usually sets this

Then build with --features mkl.
EOF
	exit 1
}
