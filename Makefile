frontend.js:;nix build&&cp -p result/* . 	# run this inside of the container
test:frontend.js;python3 -m http.server 	# run this outside of the container
clean:;rm -rf frontend* result			# run this inside of the container
nix-installer.sh:;curl --proto '=https' --tlsv1.2 -sSf -L  https://github.com/DeterminateSystems/nix-installer/releases/download/v3.23.3/nix-installer.sh > nix-installer.sh
