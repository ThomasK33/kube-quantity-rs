#!/bin/bash
set -euo pipefail

main() {
	cat <<-EOS > CHANGELOG.md
	# Changelog

	$(changelog-from-release -l 2)
	EOS
}

# shellcheck disable=SC2068
main $@
