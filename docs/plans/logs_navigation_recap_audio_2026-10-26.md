# Récap audio — Plan « Refonte des logs » (rusty-console, PR #19)

> Transcription d'un récap ~2 minutes, calibré pour une lecture à voix haute (TTS) ou écoute en podcast interne.

---

Objectif : moderniser l'écran des logs du conteneur, en s'inspirant d'oxker, tout en respectant notre architecture en quatre couches.

**Premier point : la navigation.** Aujourd'hui, les logs sont un bloc de texte avec un défilement global. Demain : une vraie liste, une ligne = une entrée, avec son timestamp Docker activé à la source. On navigue ligne par ligne avec les flèches ou j et k, plus PageUp et PageDown. On remplace donc le scroll pixel par un ListState Ratatui, comme les autres écrans.

**Deuxième point : les lignes tronquées.** Quand une ligne est trop longue, oxker fait du scroll horizontal. Nous faisons mieux : une touche ouvre une vue « ligne étendue » en superposition, avec retour à la ligne automatique. Timestamp et message complets, lisibles, fermables avec Échap. On réutilise le pattern d'overlay de la boîte de confirmation.

**Troisième point : la recherche, mode vi.** Slash ouvre une barre de saisie. Entrée valide, et n saute au résultat suivant, N majuscule au précédent, avec boucle circulaire. Les lignes trouvées sont surlignées, et le titre affiche le terme plus un compteur position sur total. Attention : slash est déjà utilisé pour filtrer sur les listes, donc le mapping des touches sera fait par écran.

**Quatre : l'export.** Touche E majuscule exporte les logs complets du conteneur — pas seulement les cinq cents lignes affichées — dans un fichier .log, nommé : nom du conteneur, date et heure, format AAAAMMJJ_HHMMSS. Écriture asynchrone via tokio, feedback à l'écran, erreur affichée si le répertoire n'est pas inscriptible.

**Côté architecture :** le parsing timestamp-message se fait dans la couche application, dans un nouveau DTO LogLine par ligne — testable sans démon Docker. C'est l'écart assumé avec oxker, qui découpe tout à l'affichage.

**Sept étapes, un commit chacune**, repo vert en continu : parsing, timestamps infra, navigation, ligne étendue, recherche, export, et quality gate final. Hors périmètre : le streaming temps réel, le filtrage masquant, et les regex.

Le plan est persisté dans docs/plans, PR #19 en draft, prêt pour l'agent développeur.
