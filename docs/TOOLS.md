# Tools

Raskolnikov wraps these external tools. Each is optional — missing tools are
reported at startup but do not prevent the agent from running.

| Tool | Purpose | Detection |
|------|---------|-----------|
| nmap | Port scanning, service detection, OS fingerprinting | `nmap --version` |
| gobuster / ffuf | Web directory and DNS brute-forcing | `gobuster --version` / `ffuf -V` |
| nikto | Web server vulnerability scanning | `nikto -Version` |
| sqlmap | SQL injection detection and exploitation | `sqlmap --version` |

## Arch Linux

```bash
sudo pacman -S nmap gobuster nikto sqlmap
```

ffuf is available from AUR:

```bash
yay -S ffuf
```

## Debian / Ubuntu / Kali

```bash
sudo apt update
sudo apt install nmap gobuster nikto sqlmap
```

ffuf is available from GitHub releases or AUR:

```bash
wget https://github.com/ffuf/ffuf/releases/latest/download/ffuf_1.5.0_linux_amd64.tar.gz
tar xzf ffuf_1.5.0_linux_amd64.tar.gz
sudo mv ffuf /usr/local/bin/
```

## Fedora

```bash
sudo dnf install nmap gobuster nikto sqlmap
```

ffuf:

```bash
sudo dnf install ffuf
```

## macOS (Homebrew)

```bash
brew install nmap gobuster nikto sqlmap
brew install ffuf
```

## NixOS

```bash
nix-shell -p nmap gobuster nikto sqlmap ffuf
```

Or add to your `configuration.nix`:

```nix
environment.systemPackages = with pkgs; [ nmap gobuster nikto sqlmap ffuf ];
```

## From source

```bash
# nmap
git clone https://github.com/nmap/nmap
cd nmap && ./configure && make && sudo make install

# gobuster (Go required)
go install github.com/OJ/gobuster/v3@latest

# ffuf (Go required)
go install github.com/ffuf/ffuf/v2@latest

# nikto (Perl required)
git clone https://github.com/sullo/nikto
cd nikto && sudo ln -s $PWD/program/nikto.pl /usr/local/bin/nikto

# sqlmap (Python required)
git clone --depth 1 https://github.com/sqlmapproject/sqlmap
sudo ln -s $PWD/sqlmap/sqlmap.py /usr/local/bin/sqlmap
```
