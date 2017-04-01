openssl req -nodes -x509 -newkey rsa:2048 -config ssl.conf -extensions ext -subj /C=CA/ST=EH/L=Canadia/O=Dis/CN=rem -keyout rem.key -out rem.crt -days 365
openssl pkcs12 -export -nodes -inkey rem.key -in rem.crt -out rem.pfx
sudo cp rem.crt /usr/local/share/ca-certificates/rem.crt 
sudo /usr/sbin/update-ca-certificates
cp -f rem.pfx ../