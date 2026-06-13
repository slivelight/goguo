import { useState } from 'react';

interface TemplateConfirmDialogProps {
  isOpen: boolean;
  template: string;
  onConfirm: () => void;
  onCancel: () => void;
}

interface SiteDetail {
  id: string;
  name: string;
  domain_count: number;
  domains: Record<string, string[]>;
}

const TEMPLATE_SITE_DETAILS: Record<string, SiteDetail[]> = {
  developer: [
    { id: 'github', name: 'GitHub', domain_count: 47, domains: { core: ['github.com', 'github.io', 'githubusercontent.com', 'githubassets.com', 'ghcr.io', 'ssh.github.com', 'lfs.github.com'], api: ['api.github.com', 'graphql.github.com', 'actions.githubusercontent.com'], assets: ['raw.githubusercontent.com', 'codeload.github.com', 'avatars.githubusercontent.com'], services: ['gist.github.com', 'copilot.github.com', 'githubcopilot.com', 'pages.github.com', 'codespaces.githubusercontent.com'], packages: ['npm.pkg.github.com', 'maven.pkg.github.com', 'nuget.pkg.github.com', 'rubygems.pkg.github.com', 'pypi.pkg.github.com'] } },
    { id: 'npmjs', name: 'npm', domain_count: 3, domains: { core: ['npmjs.com', 'registry.npmjs.org', 'static.npmjs.com'] } },
    { id: 'claude', name: 'Claude', domain_count: 15, domains: { core: ['claude.ai', 'claude.com', 'anthropic.com'], api: ['api.anthropic.com'], cdn: ['cdn.anthropic.com'], third_party: ['intercom.io', 'sentry.io', 'statsigapi.net'], cross_dependency: ['accounts.google.com', 'fonts.googleapis.com', 'fonts.gstatic.com'] } },
    { id: 'chatgpt', name: 'ChatGPT', domain_count: 23, domains: { core: ['chatgpt.com', 'openai.com', 'chat.openai.com', 'help.openai.com', 'platform.openai.com'], api: ['api.openai.com', 'auth.openai.com'], cdn: ['oaistatic.com', 'oaiusercontent.com', 'cdn.openai.com'], third_party: ['challenges.cloudflare.com', 'sentry.io', 'js.stripe.com'] } },
    { id: 'oracle', name: 'Oracle', domain_count: 4, domains: { core: ['oracle.com'], services: ['cloud.oracle.com', 'docs.oracle.com'], cdn: ['oracleimg.com'] } },
    { id: 'docker', name: 'Docker', domain_count: 4, domains: { core: ['docker.com', 'docker.io'], packages: ['registry.docker.com', 'registry.hub.docker.com'] } },
    { id: 'stackoverflow', name: 'Stack Overflow', domain_count: 10, domains: { core: ['stackoverflow.com'], cdn: ['sstatic.net', 'cdn.sstatic.net'], services: ['stackexchange.com'], cross_dependency: ['superuser.com', 'askubuntu.com', 'serverfault.com', 'stackapps.com'] } },
    { id: 'pypi', name: 'PyPI', domain_count: 3, domains: { core: ['pypi.org'], packages: ['files.pythonhosted.org'], api: ['test.pypi.org'] } },
    { id: 'crates', name: 'Crates.io', domain_count: 3, domains: { core: ['crates.io', 'static.crates.io', 'index.crates.io'] } },
  ],
  office: [
    { id: 'google', name: 'Google', domain_count: 22, domains: { core: ['google.com', 'google.com.hk', 'googleapis.com'], api: ['accounts.google.com'], cdn: ['gstatic.com', 'googleusercontent.com', 'fonts.googleapis.com', 'fonts.gstatic.com'], services: ['youtube.com', 'gmail.com', 'drive.google.com', 'maps.google.com', 'play.google.com', 'scholar.google.com'] } },
    { id: 'wikipedia', name: 'Wikipedia', domain_count: 5, domains: { core: ['wikipedia.org', 'en.wikipedia.org'], cdn: ['wikimedia.org', 'upload.wikimedia.org'], services: ['wikidata.org'] } },
    { id: 'whatsapp', name: 'WhatsApp', domain_count: 5, domains: { core: ['whatsapp.com', 'web.whatsapp.com'], cdn: ['whatsapp.net', 'static.whatsapp.net'], api: ['api.whatsapp.com'] } },
    { id: 'instagram', name: 'Instagram', domain_count: 5, domains: { core: ['instagram.com'], cdn: ['cdninstagram.com', 'fbcdn.net'], api: ['graph.instagram.com', 'api.instagram.com'] } },
    { id: 'canva', name: 'Canva', domain_count: 3, domains: { core: ['canva.com'], cdn: ['cdn.canva.com'], assets: ['static.canva.com'] } },
    { id: 'twitter-x', name: 'X (Twitter)', domain_count: 6, domains: { core: ['x.com', 'twitter.com'], cdn: ['twimg.com', 'pbs.twimg.com', 'abs.twimg.com'], api: ['api.x.com'] } },
  ],
};

function TemplateConfirmDialog({ isOpen, template, onConfirm, onCancel }: TemplateConfirmDialogProps) {
  const [expandedId, setExpandedId] = useState<string | null>(null);

  if (!isOpen) return null;

  const sites = TEMPLATE_SITE_DETAILS[template];
  if (!sites) return null;

  const label = template === 'developer' ? '开发者模板' : '办公模板';

  return (
    <div className="confirm-dialog-overlay">
      <div className="confirm-dialog" style={{ maxWidth: '560px' }}>
        <h3 className="confirm-dialog-title">应用预设模板</h3>
        <p className="confirm-dialog-message">
          <strong>{label}</strong>将添加以下 {sites.length} 个站点：
        </p>
        <ul style={{
          listStyle: 'none',
          padding: 0,
          margin: '0 0 12px',
          maxHeight: '320px',
          overflowY: 'auto',
        }}>
          {sites.map((site) => {
            const isExpanded = expandedId === site.id;
            const allDomains = Object.values(site.domains).flat();
            return (
              <li key={site.id} style={{
                padding: '8px',
                borderBottom: '1px solid var(--color-border, #eee)',
                fontSize: '14px',
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                  <div>
                    <span style={{ fontWeight: '600' }}>{site.name}</span>
                    <span style={{ fontSize: '12px', color: 'var(--color-text-secondary)', marginLeft: '8px' }}>
                      {site.domain_count} 个域名
                    </span>
                  </div>
                  <button
                    type="button"
                    className="btn btn-secondary"
                    style={{ fontSize: '11px', padding: '2px 8px' }}
                    onClick={() => setExpandedId(isExpanded ? null : site.id)}
                  >
                    {isExpanded ? '收起' : '展开'}
                  </button>
                </div>
                {isExpanded && (
                  <div style={{ marginTop: '8px', maxHeight: '120px', overflowY: 'auto' }}>
                    {allDomains.map((d) => (
                      <span
                        key={d}
                        style={{
                          display: 'inline-block',
                          padding: '1px 6px',
                          margin: '2px',
                          background: 'var(--color-bg-secondary, #f0f0f0)',
                          borderRadius: '3px',
                          fontSize: '11px',
                        }}
                      >
                        {d}
                      </span>
                    ))}
                  </div>
                )}
              </li>
            );
          })}
        </ul>
        <div className="confirm-dialog-actions">
          <button type="button" className="btn btn-primary" onClick={onConfirm}>应用</button>
          <button type="button" className="btn btn-secondary" onClick={onCancel}>取消</button>
        </div>
      </div>
    </div>
  );
}

export default TemplateConfirmDialog;
