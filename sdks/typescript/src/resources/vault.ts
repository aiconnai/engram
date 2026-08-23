import { BaseResource } from "./base.js";
import type {
  VaultExportOptions,
  VaultExportReport,
  VaultImportOptions,
  VaultImportReport,
} from "../types.js";

export class VaultResource extends BaseResource {
  /**
   * Export memories to Markdown files with YAML frontmatter (Obsidian vault compatible).
   */
  async export(options: VaultExportOptions = {}): Promise<VaultExportReport> {
    const params: Record<string, unknown> = {
      workspace: options.workspace ?? "default",
    };
    if (options.outputDir) params.output_dir = options.outputDir;
    if (options.group) params.group = options.group;
    if (options.includeLinks !== undefined)
      params.include_links = options.includeLinks;

    return (await this.caller.mcpCall(
      "memory_export_markdown",
      params
    )) as VaultExportReport;
  }

  /**
   * Import Markdown files into Engram with SHA-256 drift detection.
   * By default, runs with confirm: true unless dryRun: true is specified.
   */
  async import(options: VaultImportOptions): Promise<VaultImportReport> {
    const confirm = options.confirm ?? (options.dryRun ? false : true);
    const params: Record<string, unknown> = {
      input_dir: options.inputDir,
      confirm,
    };
    if (options.workspace) params.workspace = options.workspace;
    if (options.forceVersion !== undefined)
      params.force_version = options.forceVersion;

    return (await this.caller.mcpCall(
      "memory_import_markdown",
      params
    )) as VaultImportReport;
  }

  /**
   * Preview import without mutating the database (dry-run review mode).
   */
  async preview(
    options: Omit<VaultImportOptions, "confirm" | "dryRun">
  ): Promise<VaultImportReport> {
    return this.import({ ...options, dryRun: true, confirm: false });
  }
}
