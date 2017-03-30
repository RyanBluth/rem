openssl req -nodes -x509 -newkey rsa:2048 -config ssl.conf -extensions ext -subj /C=CA/ST=EH/L=Canadia/O=Dis/CN=remdev -keyout localhost.key -out localhost.crt -days 365
openssl pkcs12 -export -nodes -inkey localhost.key -in localhost.crt -out localhost.pfx
sudo cp localhost.crt /usr/share/ca-certificates/localhost.crt 
sudo cp localhost.key /etc/ssl/private/
sudo cp localhost.crt /etc/ssl/certs/
sudo /usr/sbin/update-ca-certificates
mv localhost.pfx ../