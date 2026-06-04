import { describe, it, expect, vi } from 'vitest';
import { addTargetSite, removeTargetSite, previewRules } from '../tauri-ipc';

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === 'add_target_site') {
      return Promise.resolve({
        success: true,
        site: { id: args?.siteId, name: 'Test', domain_count: 5, domains: {} },
        rules_generated: 10,
        verification_passed: true,
      });
    }
    if (cmd === 'remove_target_site') {
      return Promise.resolve({
        success: true,
        remaining_sites: 2,
      });
    }
    if (cmd === 'preview_rules') {
      return Promise.resolve(['DOMAIN-SUFFIX,github.com', 'DOMAIN-SUFFIX,github.io']);
    }
    return Promise.resolve({});
  }),
}));

describe('tauri-ipc', () => {
  it('addTargetSite calls invoke with correct parameters', async () => {
    const result = await addTargetSite('github');
    expect(result.success).toBe(true);
    expect(result.site?.id).toBe('github');
  });

  it('removeTargetSite calls invoke with correct parameters', async () => {
    const result = await removeTargetSite('github');
    expect(result.success).toBe(true);
    expect(result.remaining_sites).toBe(2);
  });

  it('previewRules returns array of rules', async () => {
    const result = await previewRules();
    expect(result).toHaveLength(2);
    expect(result[0]).toBe('DOMAIN-SUFFIX,github.com');
  });
});