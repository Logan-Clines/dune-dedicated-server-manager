# Manual Ubuntu Server Setup Guide

This guide explains how to install the Dune Awakening Playtest dedicated server on a fresh Ubuntu host without using Dune Dedicated Server Manager or `dune-manager-cli`.

This path uses only:

- Ubuntu shell commands
- SteamCMD
- k3s
- the server package scripts downloaded from Steam

Use a fresh server. The setup installs packages, creates a service user, downloads server files, installs k3s, imports Kubernetes resources, writes configuration, and opens game services. Do not run it on a machine that already hosts important workloads.

## 1. Requirements

Recommended host:

- Ubuntu 24.04 or newer
- x86_64 CPU
- At least 4 CPU cores
- At least 20 GiB RAM for a basic Hagga Basin/Sietch layout
- 30 GiB RAM for Hagga Basin plus Story/Social maps
- 40 GiB RAM for Hagga Basin plus Story/Social maps plus Deep Desert
- At least 100 GiB disk
- IPv4 connectivity
- Root login or a sudo-capable user

You also need:

- A Self-Host Service Token from Funcom
- The IPv4 address players will connect to
- SSH access to the Ubuntu server

Open these firewall ports:

- TCP 22 from your own IP address for SSH
- UDP 7777-7810 from any IP for game servers
- TCP 31982 from any IP for RMQ

If you later expose the Director UI or File Browser, prefer an SSH tunnel. Do not publicly expose those services.

## 2. SSH into the server

Connect as `root`, or connect as a user that can run `sudo`.

```sh
ssh root@YOUR_SERVER_IP
```

If you are not root, keep using `sudo` where the commands below require it.

## 3. Update Ubuntu and install prerequisites

```sh
export DEBIAN_FRONTEND=noninteractive

sudo apt-get update -y
sudo apt-get install -y \
  ca-certificates curl tar gzip unzip openssl util-linux iproute2 procps \
  lsb-release sudo python3 lib32gcc-s1 lib32stdc++6
```

## 4. Create the `dune` service user

```sh
sudo useradd -m -s /bin/bash dune 2>/dev/null || true
sudo mkdir -p /home/dune/.dune /home/dune/.dune/download /home/dune/Steam /home/dune/.steam
sudo chown -R dune:dune /home/dune/.dune /home/dune/Steam /home/dune/.steam
```

Allow the `dune` user to run the Kubernetes and service commands used by the vendor scripts:

```sh
echo 'dune ALL=(ALL) NOPASSWD:ALL' | sudo tee /etc/sudoers.d/dune-server >/dev/null
sudo chmod 0440 /etc/sudoers.d/dune-server
```

This is convenient for a single-purpose server. If you harden it later, make sure the vendor scripts can still run their required `sudo kubectl`, `k3s`, service, and file-copy commands.

## 5. Optional: add swap on low-memory hosts

Skip this on hosts with enough RAM. On smaller hosts, swap can help the server survive memory spikes, but performance may suffer.

This example creates a 30 GiB swapfile:

```sh
sudo fallocate -l 30G /swapfile || sudo dd if=/dev/zero of=/swapfile bs=1M count=30720 status=progress
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile

grep -q '^[[:space:]]*/swapfile[[:space:]]' /etc/fstab \
  || echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab >/dev/null
```

If k3s will run with swap enabled, configure kubelet before installing k3s:

```sh
sudo mkdir -p /etc/rancher/k3s
sudo tee /etc/rancher/k3s/kubelet-config.yaml >/dev/null <<'EOF'
apiVersion: kubelet.config.k8s.io/v1beta1
kind: KubeletConfiguration
failSwapOn: false
memorySwap:
  swapBehavior: LimitedSwap
EOF

sudo tee /etc/rancher/k3s/config.yaml >/dev/null <<'EOF'
kubelet-arg:
- config=/etc/rancher/k3s/kubelet-config.yaml
EOF
```

## 6. Install SteamCMD

```sh
tmp="$(mktemp -t steamcmd.XXXXXX.tar.gz)"
curl -fsSL 'https://steamcdn-a.akamaihd.net/client/installer/steamcmd_linux.tar.gz' -o "$tmp"
sudo -u dune tar -xzf "$tmp" -C /home/dune/Steam
rm -f "$tmp"

sudo -u dune ln -sfn /home/dune/Steam /home/dune/.steam/root
sudo -u dune ln -sfn /home/dune/Steam /home/dune/.steam/steam
```

Verify it starts:

```sh
sudo -u dune env HOME=/home/dune /home/dune/Steam/steamcmd.sh +quit
```

## 7. Download the Dune server package

The Dune dedicated server Steam app id is `3104830`.

```sh
sudo -u dune env HOME=/home/dune /home/dune/Steam/steamcmd.sh \
  +@ShutdownOnFailedCommand 1 \
  +@NoPromptForPassword 1 \
  +set_spew_level 1 1 \
  +force_install_dir /home/dune/.dune/download \
  +login anonymous \
  +app_update 3104830 validate \
  +logoff \
  +quit
```

Check that the vendor scripts downloaded:

```sh
test -f /home/dune/.dune/download/scripts/setup.sh
test -f /home/dune/.dune/download/scripts/battlegroup.sh
```

If either command fails, rerun the SteamCMD download. Steam downloads can fail transiently.

## 8. Install k3s

```sh
curl -sfL https://get.k3s.io -o /tmp/install-k3s.sh
chmod 0755 /tmp/install-k3s.sh
sudo INSTALL_K3S_EXEC='server --disable=traefik --write-kubeconfig-mode=644' sh /tmp/install-k3s.sh
rm -f /tmp/install-k3s.sh

sudo systemctl enable k3s
sudo systemctl start k3s
```

Wait for Kubernetes to become ready:

```sh
sudo kubectl get nodes
sudo kubectl wait --for=condition=Ready node --all --timeout=180s
```

## 9. Write the player-facing IP

Pick the IP address players should use. For a public VPS, this is usually the server's public IPv4 address.

```sh
PLAYER_IP="YOUR_PUBLIC_OR_PLAYER_FACING_IP"

printf '\n\n\n%s\n' "$PLAYER_IP" | sudo tee /home/dune/.dune/settings.conf >/dev/null
sudo chown dune:dune /home/dune/.dune/settings.conf
```

## 10. Run the vendor setup script

The downloaded server package owns the final Kubernetes bootstrap and world creation flow. Run it as the `dune` user:

```sh
sudo -iu dune
cd /home/dune/.dune/download
chmod +x scripts/setup.sh scripts/battlegroup.sh
./scripts/setup.sh
```

Follow the prompts from the vendor script. When asked, provide:

- Your Self-Host Service Token
- The world/server name you want players to see
- The region, usually `Europe Test` or `North America Test`
- Any layout or map choices offered by the script

Do not paste the Self-Host Service Token into shared logs, screenshots, chat, or issue reports.

After setup finishes, leave the `dune` shell:

```sh
exit
```

## 11. Install the battlegroup helper shortcut

The vendor setup normally creates this helper. Run these commands anyway if `/home/dune/.dune/bin/battlegroup` is missing or not executable.

```sh
sudo mkdir -p /home/dune/.dune/bin
sudo ln -sfn /home/dune/.dune/download/scripts/battlegroup.sh /home/dune/.dune/bin/battlegroup
sudo chmod +x /home/dune/.dune/download/scripts/battlegroup.sh
sudo chown -h dune:dune /home/dune/.dune/bin/battlegroup
```

You can now manage the server from SSH with:

```sh
sudo -iu dune
/home/dune/.dune/bin/battlegroup status
/home/dune/.dune/bin/battlegroup start
```

## 12. Verify Kubernetes resources

List battlegroup namespaces:

```sh
sudo kubectl get ns | grep '^funcom-seabass-'
```

Set variables for the namespace and battlegroup name:

```sh
NS="$(sudo kubectl get ns --no-headers -o custom-columns=NAME:.metadata.name | grep '^funcom-seabass-' | head -n1)"
BG="${NS#funcom-seabass-}"

echo "Namespace: $NS"
echo "BattleGroup: $BG"
```

Check the BattleGroup:

```sh
sudo kubectl get battlegroup "$BG" -n "$NS" -o wide
sudo kubectl get pods -n "$NS" -o wide
```

Start it if it is stopped:

```sh
sudo kubectl patch battlegroup "$BG" -n "$NS" --type=merge -p '{"spec":{"stop":false}}'
```

It can take several minutes before all pods are running and the server appears in-game.

## 13. Useful management commands

Status:

```sh
sudo -iu dune /home/dune/.dune/bin/battlegroup status
```

Start:

```sh
sudo -iu dune /home/dune/.dune/bin/battlegroup start
```

Stop:

```sh
sudo -iu dune /home/dune/.dune/bin/battlegroup stop
```

Restart:

```sh
sudo -iu dune /home/dune/.dune/bin/battlegroup restart
```

Update from Steam:

```sh
sudo -iu dune /home/dune/.dune/bin/battlegroup update
```

Export logs:

```sh
sudo -iu dune /home/dune/.dune/bin/battlegroup logs-export
sudo -iu dune /home/dune/.dune/bin/battlegroup operator-logs-export
```

Raw Kubernetes checks:

```sh
sudo kubectl get pods -A
sudo kubectl get battlegroups -A
sudo kubectl describe battlegroup "$BG" -n "$NS"
```

## 14. Access Director or File Browser safely

Do not expose Director or File Browser directly to the public internet.

For Director, first discover the NodePort:

```sh
sudo kubectl get svc -A -o jsonpath='{.items[*].spec.ports[?(@.port==11717)].nodePort}'
echo
```

From your local machine, create an SSH tunnel. Replace `DIRECTOR_NODEPORT` with the value above:

```sh
ssh -L 11717:YOUR_SERVER_IP:DIRECTOR_NODEPORT root@YOUR_SERVER_IP
```

Then open:

```text
http://127.0.0.1:11717/
```

For File Browser, the vendor management script opens TCP `18888` on the server. Tunnel it instead of opening it publicly:

```sh
ssh -L 18888:127.0.0.1:18888 root@YOUR_SERVER_IP
```

Then open:

```text
http://127.0.0.1:18888/
```

## 15. Troubleshooting

SteamCMD download fails:

```sh
sudo -u dune env HOME=/home/dune /home/dune/Steam/steamcmd.sh +quit
```

Then rerun the `app_update 3104830 validate` command.

k3s is not ready:

```sh
sudo systemctl status k3s --no-pager
sudo journalctl -u k3s -n 200 --no-pager
sudo kubectl get nodes -o wide
```

Pods are stuck:

```sh
sudo kubectl get pods -A
sudo kubectl describe pod -n "$NS" POD_NAME
sudo kubectl logs -n "$NS" POD_NAME --all-containers --tail=200
```

Operators are not ready:

```sh
sudo kubectl get pods -n funcom-operators
sudo kubectl logs -n funcom-operators deployment/battlegroupoperator-controller-manager --all-containers --tail=200
sudo kubectl logs -n funcom-operators deployment/databaseoperator-controller-manager --all-containers --tail=200
sudo kubectl logs -n funcom-operators deployment/serveroperator-controller-manager --all-containers --tail=200
sudo kubectl logs -n funcom-operators deployment/utilitiesoperator-controller-manager --all-containers --tail=200
```

The server does not appear in-game:

- Confirm UDP `7777-7810` and TCP `31982` are open in the host firewall and provider firewall.
- Confirm `PLAYER_IP` in `/home/dune/.dune/settings.conf` is the address players can actually reach.
- Confirm the BattleGroup is not stopped:

```sh
sudo kubectl get battlegroup "$BG" -n "$NS" -o jsonpath='{.spec.stop}{"\n"}'
```

If it prints `true`, start it:

```sh
sudo kubectl patch battlegroup "$BG" -n "$NS" --type=merge -p '{"spec":{"stop":false}}'
```
