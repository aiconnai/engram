import type {
  CouncilSkillAskOptions,
  CouncilSkillOptions,
  MemoryCouncilOptions,
} from "./types.js";

export interface CouncilClient {
  memoryCouncil(
    prompt: string,
    options?: MemoryCouncilOptions
  ): Promise<unknown>;
}

export class CouncilSkill {
  private readonly defaultWorkspace: string;
  private readonly defaultTimeoutSeconds: number;
  private readonly defaultIncludeRawStages: boolean;

  constructor(
    private readonly client: CouncilClient,
    options: CouncilSkillOptions = {}
  ) {
    this.defaultWorkspace = options.defaultWorkspace ?? "default";
    this.defaultTimeoutSeconds = options.defaultTimeoutSeconds ?? 90;
    this.defaultIncludeRawStages = options.defaultIncludeRawStages ?? false;
  }

  async ask(
    prompt: string,
    options: CouncilSkillAskOptions = {}
  ): Promise<unknown> {
    if (!prompt || !prompt.trim()) {
      return { error: "prompt must be a non-empty string" };
    }

    return this.client.memoryCouncil(prompt, {
      conversationId: options.conversationId,
      councilUrl: options.councilUrl,
      timeoutSeconds: options.timeoutSeconds ?? this.defaultTimeoutSeconds,
      includeRawStages:
        options.includeRawStages ?? this.defaultIncludeRawStages,
      persist: options.persist ?? false,
      workspace: options.workspace ?? this.defaultWorkspace,
      memoryTags: options.memoryTags,
    });
  }

  async askWithPersistence(
    prompt: string,
    options: Omit<CouncilSkillAskOptions, "persist"> = {}
  ): Promise<unknown> {
    return this.ask(prompt, { ...options, persist: true });
  }
}
