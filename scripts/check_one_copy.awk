# Why this gate exists: fail if any agent-ix git crate is resolved more than once
# in Cargo.lock. Two git specs of one crate (a stray rev or branch) build it twice
# with types that do not unify. Usage: awk -F'"' -f scripts/check_one_copy.awk Cargo.lock
function flush() { if (name != "") { count[name]++; if (index(src, "git+https://github.com/agent-ix/") == 1) first[name] = 1 } name = ""; src = "" }
/^\[\[package\]\]/ { flush(); next }
/^name = /   { name = $2 }
/^source = / { src = $2 }
END { flush(); bad = 0; for (n in first) if (count[n] > 1) { printf "one-copy: %s has %d entries in Cargo.lock\n", n, count[n] > "/dev/stderr"; bad = 1 } exit bad }
