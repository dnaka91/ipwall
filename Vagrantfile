Vagrant.configure("2") do |config|
  config.vm.box = "ubuntu/focal64"

  config.vm.provision "shell", inline: <<-SHELL
    apt-get update
    apt-get dist-upgrade -y
    apt-get install -y ipset xtables-addons-dkms
    apt-get autoremove -y
  SHELL
end
