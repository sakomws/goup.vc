# Custom group and event domains

Group and event managers can request one custom hostname from the relevant
dashboard page. The first release automates ownership verification but keeps TLS
and Nginx activation as an operator-controlled step.

## Lifecycle

1. The manager enters a lowercase hostname such as `events.example.org`.
2. GOUP displays a CNAME record and a unique TXT verification record.
3. The manager points a subdomain CNAME to `goup.vc` and publishes the TXT
   record. Zone-apex domains require a provider-supported ALIAS/ANAME or
   equivalent flattened CNAME.
4. The manager selects **Verify DNS**.
5. An operator provisions TLS and Nginx, tests the hostname, and activates it in
   the database.

A hostname is routed only after both DNS verification and operator activation.
Login, registration, dashboard, and state-changing requests redirect to the
canonical `https://goup.vc` host.

## EC2 activation

Confirm that the domain is verified:

```sql
select hostname, verified_at, activated_at
from custom_domain
where hostname = 'events.example.org';
```

The hostname must resolve to the GOUP edge or EC2 instance. When Cloudflare is
used, temporarily select **DNS only** while issuing the origin certificate.

Issue the certificate:

```bash
sudo certbot certonly --webroot \
  -w /var/www/certbot \
  --deploy-hook "systemctl reload nginx" \
  -d events.example.org
```

Create `/etc/nginx/conf.d/events.example.org.conf`:

```nginx
server {
    listen 80;
    server_name events.example.org;

    location /.well-known/acme-challenge/ {
        root /var/www/certbot;
    }

    location / {
        return 301 https://$host$request_uri;
    }
}

server {
    listen 443 ssl;
    http2 on;
    server_name events.example.org;

    ssl_certificate /etc/letsencrypt/live/events.example.org/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/events.example.org/privkey.pem;
    include /etc/letsencrypt/options-ssl-nginx.conf;
    ssl_dhparam /etc/letsencrypt/ssl-dhparams.pem;

    location / {
        proxy_pass http://127.0.0.1:9000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Validate and reload Nginx:

```bash
sudo nginx -t
sudo systemctl reload nginx
curl --resolve events.example.org:443:127.0.0.1 \
  https://events.example.org/ -I
```

Activate routing only after the HTTPS smoke test succeeds:

```sql
select mark_custom_domain_active(custom_domain_id)
from custom_domain
where hostname = 'events.example.org';
```

Allow up to 60 seconds for application host-routing caches to refresh.

## Removal

Remove the assignment in the dashboard before removing its Nginx configuration.
Then validate and reload Nginx:

```bash
sudo rm /etc/nginx/conf.d/events.example.org.conf
sudo nginx -t
sudo systemctl reload nginx
```

Certbot may retain an unused certificate lineage. Remove it only after confirming
that no Nginx configuration references it:

```bash
sudo certbot delete --cert-name events.example.org
```

## Renewal

Custom-domain certificates use the existing Certbot renewal timer. The deploy
hook installed above reloads Nginx only after a successful renewal. Periodically
verify all lineages:

```bash
sudo certbot renew --dry-run
```
