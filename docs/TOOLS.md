# Tools

Raskolnikov wraps these external tools. Each is optional — missing tools are
reported at startup but do not prevent the agent from running.

> **Note:** nikto requires the Perl module `XML::Writer`. Package managers
> handle this automatically; for manual/source installs, install it separately
> (e.g. `sudo perl -MCPAN -e 'install XML::Writer'`).

## Detection

Raskolnikov checks each tool at startup with its version/banner flag and lists
what it finds. Tools marked `Optional` are common in CTF labs but not required —
the agent simply skips steps that need them when absent.

| Tool | Purpose | Detection | Shipment |
|------|---------|-----------|----------|
| nmap | Port scanning, service detection, OS fingerprinting | `nmap --version` | Core |
| gobuster / ffuf | Web directory and DNS brute-forcing | `gobuster --version` / `ffuf -V` | Core |
| nikto | Web server vulnerability scanning | `nikto -Version` (requires Perl `XML::Writer`) | Core |
| sqlmap | SQL injection detection and exploitation | `sqlmap --version` | Core |
| hydra | Online password brute-force (SSH/FTP/HTTP/MySQL) | `hydra -h` (exit 255, output accepted) | Core |
| whatweb | Web technology fingerprinting | `whatweb --version` | Core |
| john | Offline hash cracking | `john` (no args — banner on stderr) | Core |
| hashcat | GPU-accelerated hash cracking | `hashcat --version` | Core |
| feroxbuster | Recursive directory brute-forcing (JSON output) | `feroxbuster --version` | Optional |
| netexec | SMB/AD enumeration and credential validation | `nxc --version` / `netexec --version` | Optional |
| nuclei | Template-based vulnerability scanning | `nuclei -version` | Optional |
| enum4linux-ng | SMB/Windows enumeration (users, shares, sessions) | `enum4linux-ng -h` | Optional |
| httpx | Fast HTTP probing at scale | `httpx -version` / `httpx-toolkit -version` (Arch) | Optional |
| smbclient | SMB share listing | `smbclient -V` | Optional |
| smbmap | SMB share + permission enumeration | `smbmap -h` | Optional |
| wpscan | WordPress vulnerability scanner | `wpscan --version` | Optional |
| masscan | Fast port scanning | `masscan --version` | Optional |
| nbtscan | NetBIOS name enumeration | `nbtscan -v` | Optional |
| dnsx | DNS resolution/probing | `dnsx -version` | Optional |
| subfinder | Subdomain enumeration | `subfinder -version` | Optional |
| impacket | AD attacks: GetNPUsers, GetUserSPNs, secretsdump | `secretsdump.py -h` / `GetNPUsers.py -h` | Optional |

"Core" tools are declared as package dependencies where the distro provides
them. "Optional" tools are best-effort — install them manually or via the
tools' own package managers (Kali provides all of them).

## Arch Linux

```bash
sudo pacman -S nmap gobuster nikto sqlmap hydra whatweb john hashcat smbclient smbmap wpscan masscan nbtscan impacket
```

Optional tools (AUR / go install):

```bash
yay -S ffuf feroxbuster netexec nuclei httpx enum4linux-ng
go install github.com/projectdiscovery/dnsx/cmd/dnsx@latest
go install github.com/projectdiscovery/subfinder/v2/cmd/subfinder@latest
```

## Kali / Debian / Ubuntu

```bash
sudo apt update
sudo apt install nmap gobuster nikto sqlmap hydra whatweb john hashcat smbclient smbmap wpscan masscan nbtscan
```

On Kali, the remaining tools are available directly:

```bash
sudo apt install feroxbuster netexec nuclei httpx enum4linux-ng ffuf dnsx subfinder
sudo pipx install netexec
```

On stock Debian/Ubuntu, install the Optional tools from their upstream releases
(see "From source" below).

## Fedora

```bash
sudo dnf install nmap gobuster nikto sqlmap hydra whatweb john hashcat smbclient wpscan masscan nbtscan
sudo dnf install ffuf feroxbuster nuclei httpx enum4linux-ng netexec subfinder
```

## macOS (Homebrew)

```bash
brew install nmap gobuster nikto sqlmap hydra whatweb john hashcat smbclient wpscan masscan
brew install feroxbuster netexec nuclei httpx enum4linux-ng ffuf dnsx subfinder impacket
```

## NixOS

```bash
nix-shell -p nmap gobuster nikto sqlmap hydra whatweb john hashcat feroxbuster netexec nuclei enum4linux-ng httpx ffuf smbclient wpscan masscan nbtscan dnsx subfinder impacket
```

Or add to your `configuration.nix`:

```nix
environment.systemPackages = with pkgs; [
  nmap gobuster nikto sqlmap hydra whatweb john hashcat
  feroxbuster netexec nuclei enum4linux-ng httpx ffuf
  smbclient wpscan masscan nbtscan dnsx subfinder impacket
];
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

# nikto (Perl + XML::Writer required)
git clone https://github.com/sullo/nikto
cd nikto && sudo ln -s $PWD/program/nikto.pl /usr/local/bin/nikto
# Also install the XML::Writer Perl module:
sudo perl -MCPAN -e 'install XML::Writer'

# sqlmap (Python required)
git clone --depth 1 https://github.com/sqlmapproject/sqlmap
sudo ln -s $PWD/sqlmap/sqlmap.py /usr/local/bin/sqlmap

# feroxbuster (Rust required)
cargo install feroxbuster

# nuclei / httpx (Go required, ProjectDiscovery)
go install github.com/projectdiscovery/nuclei/v3/cmd/nuclei@latest
go install github.com/projectdiscovery/httpx/cmd/httpx@latest

# netexec (Python required)
python3 -m pipx install netexec
# or: pip3 install --user netexec

# enum4linux-ng (Python required)
git clone https://github.com/cddmp/enum4linux-ng
sudo ln -s $PWD/enum4linux-ng/enum4linux-ng.py /usr/local/bin/enum4linux-ng

# dnsx / subfinder (Go required, ProjectDiscovery)
go install github.com/projectdiscovery/dnsx/cmd/dnsx@latest
go install github.com/projectdiscovery/subfinder/v2/cmd/subfinder@latest

# impacket (Python required — GetNPUsers.py, secretsdump.py, etc.)
git clone https://github.com/fortra/impacket
cd impacket
sudo python3 setup.py install
# or via pip: pip3 install --user impacket
```

## Docker

The published image bundles the Core tools (nmap, gobuster, sqlmap, hydra,
whatweb, john, hashcat). Install Optional tools inside a container with:

```bash
docker exec -it <container> apt-get install -y feroxbuster netexec nuclei httpx enum4linux-ng smbclient smbmap wpscan masscan nbtscan
```
