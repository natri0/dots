# Edit this configuration file to define what should be installed on
# your system. Help is available in the configuration.nix(5) man page, on
# https://search.nixos.org/options and in the NixOS manual (`nixos-help`).

{ config, lib, pkgs, ... }:

{
  imports =
    [ # Include the results of the hardware scan.
      ./hardware-configuration.nix
    ];

  boot.loader.grub.enable = true;
  boot.loader.grub.device = "/dev/vda"; # or "nodev" for efi only

  networking.hostName = "avalanche"; # Define your hostname.

  # Set your time zone.
  time.timeZone = "Europe/Kyiv";

  users.mutableUsers = false; # we configure users here
  users.users.lina = {
    isNormalUser = true;
    extraGroups = [ "wheel" "podman" ];

    hashedPassword = "$y$j9T$Mnc4VHQ4i0b85CQxFVaXB/$2XncxIjRhh6Zocl7/lfqeS8D14Mpdqg9deVaiI7lWcA";
    openssh.authorizedKeys.keys = [
      "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIJ/2Uzbkvr4NmrqlvOL+/S69NNPDIj1Ya5ORrkdBgmti laptop@Linas-MacBook-Pro.local"
    ];

    packages = with pkgs; [
    ];
  };

  # programs.firefox.enable = true;

  environment.systemPackages = with pkgs; [
    nano
    git
  ];

  # Enable the OpenSSH daemon.
  services.openssh = {
    enable = true;
    settings = {
      PasswordAuthentication = false;
      PermitRootLogin = "no";
      AllowAgentForwarding = true;
    };
  };

  virtualisation = {
    containers.enable = true;
    podman = {
      enable = true;
      dockerCompat = true;
      defaultNetwork.settings.dns_enabled = true;
    };

    oci-containers.backend = "podman";
  };

  services.caddy.enable = true;

  # Kanidm SSO
  virtualisation.oci-containers.containers.sso = {
    image = "docker.io/kanidm/server:latest";
    volumes = [
      "/var/lib/kanidm:/data"
      "${./kanidm.toml}:/data/server.toml"
    ];
    ports = [ "127.0.0.1:8001:8443" ];
  };
  services.caddy.virtualHosts."sso.natri.fyi".extraConfig = ''
    reverse_proxy 127.0.0.1:8001 {
      transport http { tls_insecure_skip_verify }
    }
    # respond "OK. check again later"
  '';
  systemd.tmpfiles.rules = [ "d /var/lib/kanidm 0750 root root -" ];

  # Open ports in the firewall.
  networking.firewall.allowedTCPPorts = [ 80 443 ];
  # networking.firewall.allowedUDPPorts = [ ... ];
  # Or disable the firewall altogether.
  # networking.firewall.enable = false;

  # Copy the NixOS configuration file and link it from the resulting system
  # (/run/current-system/configuration.nix). This is useful in case you
  # accidentally delete configuration.nix.
  # system.copySystemConfiguration = true;

  # This option defines the first version of NixOS you have installed on this particular machine,
  # and is used to maintain compatibility with application data (e.g. databases) created on older NixOS versions.
  #
  # Most users should NEVER change this value after the initial install, for any reason,
  # even if you've upgraded your system to a new NixOS release.
  #
  # This value does NOT affect the Nixpkgs version your packages and OS are pulled from,
  # so changing it will NOT upgrade your system - see https://nixos.org/manual/nixos/stable/#sec-upgrading for how
  # to actually do that.
  #
  # This value being lower than the current NixOS release does NOT mean your system is
  # out of date, out of support, or vulnerable.
  #
  # Do NOT change this value unless you have manually inspected all the changes it would make to your configuration,
  # and migrated your data accordingly.
  #
  # For more information, see `man configuration.nix` or https://nixos.org/manual/nixos/stable/options#opt-system.stateVersion .
  system.stateVersion = "25.05"; # Did you read the comment?

}

