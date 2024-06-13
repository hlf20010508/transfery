#! /bin/sh

# not work on alpine
# version=$(wget --max-redirect=0 -O /dev/null https://github.com/hlf20010508/transfery-vue/releases/latest 2>&1 | grep "Location" | sed -E 's/.*\/tag\/([^ ]+).*/\1/')

version=$(curl -Ls -o /dev/null -w %{url_effective} https://github.com/hlf20010508/transfery-vue/releases/latest | awk -F'/' '{print $NF}')
git clone --depth=1 -b $version https://github.com/hlf20010508/transfery-vue.git
