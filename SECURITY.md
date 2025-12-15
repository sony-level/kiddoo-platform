# Politique de Sécurité - API Babysitting Service (Rust/Microservices)

## Versions Supportées

| Version | Statut          | Fin du Support |
| ------- | --------------- | -------------- |
| 1.x     | :white_check_mark: Actif | Décembre 2026 |
| < 1.0   | :x: Non supporté | - |

## Signaler une Vulnérabilité

Nous prenons la sécurité de notre API très au sérieux. Si vous découvrez une vulnérabilité de sécurité, nous vous prions de nous en faire part de manière responsable.

### Comment signaler une vulnérabilité

1. **Contact** : Envoyez un email à [security@babysitting-service.com](mailto:no-reply@ginsmx.com) avec le sujet "[Sécurité] Description de la vulnérabilité"
2. **Délai de réponse** : Vous recevrez un accusé de réception sous 48 heures
3. **Confidentialité** : Nous traitons tous les rapports de manière confidentielle

### Ce qu'il faut inclure dans votre rapport

- Description détaillée de la vulnérabilité
- Étapes pour reproduire le problème
- Impact potentiel
- Toute preuve de concept ou code d'exploitation
- Vos coordonnées pour un suivi

## Processus de Gération des Vulnérabilités

1. **Accusé de réception** : Sous 48 heures
2. **Enquête** : Notre équipe examinera le rapport sous 5 jours ouvrables
3. **Mise à jour** : Vous serez tenu informé de l'avancement
4. **Résolution** : Un correctif sera publié dans les plus brefs délais
5. **Divulgation** : Après résolution, nous publierons un avis de sécurité

## Architecture et Sécurité

Cette API est développée en Rust et suit une architecture microservices. Voici les aspects clés de sécurité liés à cette architecture :

- **Isolation des services** : Chaque microservice est indépendant et isolé
- **Communication sécurisée** : Toutes les communications entre services utilisent des canaux sécurisés (HTTPS, gRPC avec TLS)
- **Gestion des secrets** : Les informations sensibles sont gérées via des variables d'environnement et des secrets
- **Conteneurisation** : Les services sont conteneurisés avec des politiques de sécurité renforcées

## Bonnes Pratiques de Sécurité

### Pour les Utilisateurs
- Ne partagez jamais vos identifiants de connexion
- Utilisez des mots de passe forts et uniques
- Activez l'authentification à deux facteurs si disponible

### Pour les Développeurs
- Toutes les communications doivent utiliser HTTPS
- Validez et assainissez toutes les entrées utilisateur
- Implémentez le principe du moindre privilège
- Maintenez les dépendances à jour

## Politique de Divulgation

- Les vulnérabilités seront rendues publiques une fois corrigées
- Les contributeurs seront crédités si désiré
- Un CVE sera demandé pour les vulnérabilités critiques

## Contact d'Urgence

Pour les problèmes de sécurité urgents en dehors des heures de bureau, veuillez contacter :
- Téléphone : +33 1 23 45 67 89 (disponible 24/7 pour les urgences critiques)
- Email : [emergency@babysitting-service.com](mailto:emergency@babysitting-service.com)

---

# Security Policy - Babysitting Service API (Rust/Microservices)

## Supported Versions

| Version | Status           | End of Support |
| ------- | ---------------- | -------------- |
| 1.x     | :white_check_mark: Active | December 2026 |
| < 1.0   | :x: Not Supported | - |

## Reporting a Vulnerability

We take the security of our API very seriously. If you discover a security vulnerability, we ask you to report it to us responsibly.

### How to Report a Vulnerability

1. **Contact**: Email [security@babysitting-service.com](mailto:no-reply@ginsmx.co) with the subject "[Security] Vulnerability Description"
2. **Response Time**: You will receive an acknowledgment within 48 hours
3. **Confidentiality**: All reports are handled confidentially

### What to Include in Your Report

- Detailed description of the vulnerability
- Steps to reproduce the issue
- Potential impact
- Any proof-of-concept or exploit code
- Your contact information for follow-up

## Vulnerability Management Process

1. **Acknowledgment**: Within 48 hours
2. **Investigation**: Our team will review the report within 5 business days
3. **Updates**: You will be kept informed of progress
4. **Resolution**: A fix will be released as soon as possible
5. **Disclosure**: After resolution, we will publish a security advisory

## Architecture and Security

This API is developed in Rust and follows a microservices architecture. Here are the key security aspects of this architecture:

- **Service Isolation**: Each microservice is independent and isolated
- **Secure Communication**: All inter-service communications use secure channels (HTTPS, gRPC with TLS)
- **Secret Management**: Sensitive information is managed via environment variables and secrets
- **Containerization**: Services are containerized with enhanced security policies

## Security Best Practices

### For Users
- Never share your login credentials
- Use strong, unique passwords
- Enable two-factor authentication if available

### For Developers
- All communications must use HTTPS
- Validate and sanitize all user inputs
- Implement the principle of least privilege
- Keep dependencies up to date

## Disclosure Policy

- Vulnerabilities will be made public once fixed
- Contributors will be credited if desired
- CVEs will be requested for critical vulnerabilities

## Emergency Contact

For urgent security issues outside business hours, please contact:
- Phone:
- Email: [emergency@babysitting-service.com](mailto:no-reply@ginsmx.co)
