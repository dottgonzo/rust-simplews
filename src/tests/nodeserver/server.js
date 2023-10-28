const fs = require('fs');
const https = require('https');
const WebSocket = require('ws');

// Carica i certificati
const passphrase = 'test';
const privateKey = fs.readFileSync('key.pem', 'utf8');
const certificate = fs.readFileSync('cert.pem', 'utf8');
const credentials = { key: privateKey, cert: certificate, passphrase };

const httpsServer = https.createServer(credentials, (req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('WebSocket Server HTTPS in esecuzione');
});

const wss = new WebSocket.Server({ server: httpsServer });

wss.on('connection', (ws) => {
  console.log('Cliente connesso');

  ws.on('message', (message) => {
    console.log(`Ricevuto: ${message}`);
  });
  ws.send('Benvenuto al server WebSocket HTTPS!');

});

httpsServer.listen(3000, () => {
  console.log('Server HTTPS in ascolto sulla porta 3000');
});
