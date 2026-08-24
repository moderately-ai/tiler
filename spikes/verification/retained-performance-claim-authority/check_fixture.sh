#!/bin/sh
set -eu

claims=${1:?usage: check_fixture.sh OWNER_CLAIMS_TSV}
dispositions=$(dirname "$0")/profile-dispositions.tsv

awk -F '\t' '
FILENAME == ARGV[1] {
    if (FNR == 1) {
        if ($0 != "claim_id\towner\trevision") {
            print "invalid owner-claim header" > "/dev/stderr"
            exit 1
        }
        next
    }
    if (NF != 3 || $1 == "" || $2 == "" || $3 !~ /^[1-9][0-9]*$/) {
        print "invalid owner claim at line " FNR > "/dev/stderr"
        exit 1
    }
    if ($1 in claims) {
        print "duplicate performance claim: " $1 > "/dev/stderr"
        exit 1
    }
    claims[$1] = 1
    claim_count++
    next
}
FILENAME == ARGV[2] {
    if (FNR == 1) {
        if ($0 != "claim_id\tdisposition") {
            print "invalid disposition header" > "/dev/stderr"
            exit 1
        }
        next
    }
    if (NF != 2 || $1 == "" || ($2 != "required" && $2 != "excluded")) {
        print "invalid disposition at line " FNR > "/dev/stderr"
        exit 1
    }
    if ($1 in seen_disposition) {
        print "duplicate performance disposition: " $1 > "/dev/stderr"
        exit 1
    }
    if (!($1 in claims)) {
        print "disposition for unknown performance claim: " $1 > "/dev/stderr"
        exit 1
    }
    seen_disposition[$1] = 1
    disposition_count++
}
END {
    if (claim_count == 0) {
        print "owner manifest reached zero claims" > "/dev/stderr"
        exit 1
    }
    for (claim in claims) {
        if (!(claim in seen_disposition)) {
            print "undisposed performance claim: " claim > "/dev/stderr"
            exit 1
        }
    }
    print claim_count " claims; " disposition_count " dispositions; complete"
}
' "$claims" "$dispositions"
