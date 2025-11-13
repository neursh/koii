# Koii
A privacy-friendly thingy to handle authentication for my projects and an OAuth2 provider.

This auth server will allow people to use my projects without having to create an account on each of my future projects. While at it, I'll make it available to third-party services to use Koii as an OAuth2 method.

By default for third-party services, no additional information will be given when connecting, a new ID ties to that service will be issued on your Koii account, and that's the only thing the service will know about you, a unique ID for that service alone.

Services can request for your email, and they have to explicitly disclose reasons why in the OAuth screen. It's also expected that ALL features from that service MUST work without the need for your emails

Man these are some bold claims like ppl would use this ToT

## Rate limits
Rate limits is configured by tightrope, a load balancer to hold everything together.

This rate limits apply for unauthenticated users:
- **Signing up**: 10 accounts can be created from a single IPv4/v6 address every 3 hours.
- **Loging in**: 20 login operations can be made from a single IPv4/v6 address every 2 hours.

For authenticated users, there will be rate limits too, though it won't have much of an impact if you don't do anything crazy, more details later as I build this thing.
