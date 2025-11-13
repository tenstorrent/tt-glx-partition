# TT-GLX-ETH-ISOLATION

This tool is used to disable and restore the eth links in the tenstorrent galaxy systems. This allows users to cleanly partition the galaxy into easily virtualized and containerized sections.

Currently it has a CLI that supports -
- Isolating all UBB trays from each other
- Isolating one UBB tray from the rest
- Restoring all UBB trays to be connected to each other
- Restoring one UBB tray to the default eth ports enabled

# Install

If rust is not installed on your system, please run the following -
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

Then, clone the repo and then run `cargo run -- <ARGS>` to run the program.
All the options are as follows

```
sbansal@yyz-ubb-02:~/tt-glx-eth-isolation$ cargo run -- --help
   Compiling tt-glx-eth-isolation v0.1.0 (/home/sbansal/tt-glx-eth-isolation)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.41s
     Running `target/debug/tt-glx-eth-isolation --help`
This tool is used for BH and WH Tenstorrent Galaxy devices to eth isolate chips into custom arrangements.

Usage: tt-glx-eth-isolation [OPTIONS]

Options:
      --isolate-all-trays            Isolate all UBB trays from each other
      --isolate-tray <ISOLATE_TRAY>  Isolate a specific UBB tray from others. Provide UBB number (0-indexed)
      --restore-all-trays            Restore all UBB trays to default Ethernet settings
      --restore-tray <RESTORE_TRAY>  Restore a specific UBB tray to default Ethernet settings. Provide UBB number (0-indexed)
      --gui                          Launch the GUI interface. (Does not work currently - just a template for now)
  -h, --help                         Print help
  -V, --version                      Print version
```

## License

This project is licensed under [Apache License 2.0](./LICENSE). You have
permission to use this code under the conditions of the license pursuant to the
rights it grants.

This software assists in programming Tenstorrent products. Making, using, or
selling hardware, models, or IP may require the license of rights (such as
patent rights) from Tenstorrent or others.
