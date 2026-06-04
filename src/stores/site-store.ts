import { create } from 'zustand';
import type { SiteInfo, AddSiteResponse, RemoveSiteResponse, TemplateResponse, SiteReachability, SiteDefinitionInfo } from '../lib/types';
import { addTargetSite, removeTargetSite, applyPresetTemplate, getSiteReachability, listSiteDefinitions } from '../lib/tauri-ipc';

interface SiteState {
  sites: SiteInfo[];
  siteDefinitions: SiteDefinitionInfo[];
  reachability: SiteReachability[];
  isLoading: boolean;
  error: string | null;
}

interface SiteActions {
  fetchSites: () => Promise<void>;
  addSite: (siteId: string) => Promise<AddSiteResponse>;
  removeSite: (siteId: string) => Promise<RemoveSiteResponse>;
  applyTemplate: (template: string) => Promise<TemplateResponse>;
  fetchReachability: () => Promise<void>;
  reset: () => void;
}

const initialState: SiteState = {
  sites: [],
  siteDefinitions: [],
  reachability: [],
  isLoading: false,
  error: null,
};

export const useSiteStore = create<SiteState & SiteActions>((set) => ({
  ...initialState,

  fetchSites: async () => {
    set({ isLoading: true, error: null });
    try {
      const [reachability, definitions] = await Promise.all([
        getSiteReachability(),
        listSiteDefinitions(),
      ]);
      const defMap = new Map(definitions.map(d => [d.id, d]));
      const sites = reachability.sites.map((r) => {
        const def = defMap.get(r.site_id);
        return def
          ? { id: def.id, name: def.name, domain_count: def.domain_count, domains: def.domains }
          : { id: r.site_id, name: r.site_id, domain_count: 0, domains: {} };
      });
      set({
        sites,
        siteDefinitions: definitions,
        reachability: reachability.sites,
        isLoading: false,
      });
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to fetch sites',
      });
    }
  },

  addSite: async (siteId: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await addTargetSite(siteId);
      if (response.success && response.site) {
        set((state) => ({
          sites: [...state.sites, response.site!],
          isLoading: false,
        }));
      } else {
        set({
          isLoading: false,
          error: response.error || 'Failed to add site',
        });
      }
      return response;
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to add site',
      });
      return {
        success: false,
        rules_generated: 0,
        verification_passed: false,
        error: err instanceof Error ? err.message : 'Failed to add site',
      };
    }
  },

  removeSite: async (siteId: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await removeTargetSite(siteId);
      if (response.success) {
        set((state) => ({
          sites: state.sites.filter((s) => s.id !== siteId),
          reachability: state.reachability.filter((r) => r.site_id !== siteId),
          isLoading: false,
        }));
      } else {
        set({
          isLoading: false,
          error: response.error || 'Failed to remove site',
        });
      }
      return response;
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to remove site',
      });
      return {
        success: false,
        remaining_sites: 0,
        error: err instanceof Error ? err.message : 'Failed to remove site',
      };
    }
  },

  applyTemplate: async (template: string) => {
    set({ isLoading: true, error: null });
    try {
      const response = await applyPresetTemplate(template);
      set({ isLoading: false });
      return response;
    } catch (err) {
      set({
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to apply template',
      });
      return {
        added_count: 0,
        failed_count: 0,
        sites: [],
      };
    }
  },

  fetchReachability: async () => {
    try {
      const reachability = await getSiteReachability();
      set({ reachability: reachability.sites });
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Failed to fetch reachability',
      });
    }
  },

  reset: () => set(initialState),
}));