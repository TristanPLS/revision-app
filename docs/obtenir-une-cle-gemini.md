# Obtenir une clé IA gratuite (Google AI Studio) — pas à pas

L'application a besoin d'une « clé API » pour que l'intelligence artificielle
puisse transformer tes cours en flashcards, examens, etc. Une clé, c'est juste un
long mot de passe que **toi seul** possèdes.

**Bonne nouvelle : avec Google, c'est 100 % gratuit.** Le modèle utilisé (Gemma 4)
permet environ **1500 générations par jour** — bien plus que ce dont tu auras besoin
pour réviser. Pas de carte bancaire, pas d'abonnement.

> ⏱️ Compte 2 minutes. Il te faut juste un compte Google (Gmail).

## Étapes

1. Va sur **[aistudio.google.com/apikey](https://aistudio.google.com/apikey)**.
2. Connecte-toi avec ton compte Google si ce n'est pas déjà fait.
3. Clique sur le bouton **« Create API key »** (Créer une clé API).
   - Si on te demande de choisir ou créer un « projet », accepte celui proposé
     par défaut — peu importe son nom.
4. Une longue suite de lettres et de chiffres apparaît (elle commence par `AIza…`).
   Clique sur **« Copy »** (Copier).
5. Reviens dans l'application → page **Réglages** → colle la clé dans le champ
   **« Clé API »** → clique sur **« Enregistrer et tester »**.
6. Si tu vois ✅ **« Connexion réussie »**, c'est bon — tu peux générer tes
   premiers supports de révision ! 🎉

## Questions fréquentes

**C'est vraiment gratuit ?**
Oui. Le palier gratuit de Google AI Studio ne demande aucune carte bancaire. La
limite (~1500 requêtes/jour) se réinitialise chaque jour.

**Ma clé est-elle en sécurité ?**
Elle est stockée uniquement sur **ton** serveur (l'ordinateur où tourne
l'application), jamais envoyée à quelqu'un d'autre que Google. Ne la partage avec
personne — c'est comme un mot de passe.

**Que part-il chez Google ?**
Quand tu génères des supports, le **texte de ton cours** est envoyé à Google pour
être analysé. Avec le palier gratuit, Google peut s'en servir pour améliorer ses
produits. Évite donc d'y coller des informations vraiment confidentielles. Pour une
confidentialité totale (rien ne quitte ta machine), tu peux utiliser **Ollama** en
local — voir le [README](../README.md#-confidentialité).

**Et si je préfère Claude ou ChatGPT ?**
C'est possible (page Réglages → fournisseur **Anthropic** ou **OpenAI**), mais ces
services sont **payants à l'usage** : chaque génération coûte quelques centimes. Pour
réviser au quotidien, Google AI Studio gratuit suffit largement.

**J'ai dépassé la limite du jour ?**
Rare, mais ça peut arriver un jour de révision intense. Attends le lendemain (la
limite se réinitialise), ou passe temporairement à un autre fournisseur.
