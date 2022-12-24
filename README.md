# joincd
jumbort Open Infrastructure for Network Computing deamon.

Or more easily said: this is the server deamon for distributed computing with [jumboRT](https://github.com/jumboRT/jumboRT).
It is named after [BOINC](https://en.wikipedia.org/wiki/Berkeley_Open_Infrastructure_for_Network_Computing).

## Usage
To compile and run the deamon
`cargo run --release`

This will start the deamon, running on port 29300, which is the default port for distributed computing with jumboRT.

Once started, you will be able to connect a viewer and worker clients to it as explained in the
[jumboRT](https://github.com/jumboRT/jumboRT) repository.
