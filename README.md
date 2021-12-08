# Setup
This program is meant to be run in the https://github.com/PlummersSoftwareLLC/Primes repo.

git clone https://github.com/PlummersSoftwareLLC/Primes.git

cd Primes

cp /path/to/plummerprimes/binary ./

# Running the program

USAGE:

    plummerprimes [FLAGS] [OPTIONS]

FLAGS:

        --debug                 Debug

    -h, --help                  Prints help information

    -l, --list-formatters       List formats to output to

        --only-output-report    Only output report

    -u, --unconfined            Run with seccomp:unconfined (native performance for interpreted languages)

    -V, --version               Prints version information

OPTIONS:

    -f, --formatter <formatter>        Output formatter [default: table]

    -b, --report-base <report-base>    Report file base name [default: report]

    -r, --report-dir <report-dir>      Write report file(s) to given file [default: ./]

    -s, --save-file <save-file>        Write save to given file [default: save.db]