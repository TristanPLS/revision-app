# Politique de sécurité

## Modèle de menace (à connaître avant de signaler)

revision-app est une application **auto-hébergée mono-utilisateur, sans
authentification par design**. Le périmètre de sécurité supposé est :

- l'app n'est joignable que depuis `localhost`, un réseau de confiance ou un
  VPN (Tailscale) — jamais l'internet public ;
- la base de données n'est pas exposée hors du réseau Docker ;
- la clé API IA est stockée en clair dans la base locale (documenté).

« N'importe qui sur le réseau peut lire/écrire les données » n'est donc pas une
faille : c'est le modèle documenté. En revanche, **sont des failles** : une fuite
de la clé API (logs, réponses HTTP), une injection SQL, un XSS, un dépassement du
périmètre documenté (ex. la BDD accessible depuis l'extérieur malgré la config
par défaut), ou une vulnérabilité dans la chaîne de build/images.

## Signaler une vulnérabilité

Utilise les **GitHub Security Advisories** (onglet *Security* → *Report a
vulnerability*) plutôt qu'une issue publique. Décris le scénario d'attaque et,
si possible, une reproduction. Réponse sous quelques jours dans la mesure du
possible — c'est un projet maintenu sur temps personnel.

## Versions supportées

Seule la dernière version publiée (`main` / dernier tag) reçoit des correctifs.
