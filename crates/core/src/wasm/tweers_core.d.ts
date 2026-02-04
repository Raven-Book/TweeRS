/* tslint:disable */
/* eslint-disable */

/**
 * Input source for Twee compilation
 */
export type JsInputSource =
  | { type: "text"; name: string; content: string }
  | { type: "bytes"; name: string; data: Uint8Array; mime_type?: string };

/**
 * Story format information
 */
export interface JsStoryFormatInfo {
  name: string;
  version: string;
  source: string;
}

/**
 * Build configuration
 */
export interface JsBuildConfig {
  sources: JsInputSource[];
  format_info?: JsStoryFormatInfo;
  is_debug?: boolean;
  start_passage?: string;
}

/**
 * Passage data
 */
export interface JsPassage {
  name: string;
  tags?: string;
  position?: string;
  size?: string;
  content: string;
  source_file?: string;
  source_line?: number;
}

/**
 * Story metadata
 */
export interface JsStoryData {
  name?: string;
  ifid: string;
  format: string;
  "format-version": string;
  start?: string;
  "tag-colors"?: Record<string, string>;
  zoom?: number;
}

/**
 * Parse output containing passages and story data
 */
export interface JsParseOutput {
  passages: Record<string, JsPassage>;
  story_data: JsStoryData;
  format_info: JsStoryFormatInfo;
  is_debug: boolean;
}

/**
 * Build output containing generated HTML
 */
export class JsBuildOutput {
  free(): void;
  constructor(html: string);
  readonly html: string;
}

/**
 * Build a Twee story from the given configuration
 *
 * @param config_js - Build configuration
 * @returns Build output containing HTML
 * @throws Error if build fails
 */
export function build(config_js: JsBuildConfig): JsBuildOutput;

/**
 * Parse Twee sources without building HTML
 *
 * @param sources_js - Array of input sources
 * @returns Parsed passages, story data, and format info (with empty source)
 * @throws Error if parsing fails
 */
export function parse(sources_js: JsInputSource[]): JsParseOutput;

/**
 * Build HTML from already parsed data
 *
 * @param parsed_js - Parsed data with format_info.source filled in
 * @returns Build output containing HTML
 * @throws Error if build fails
 */
export function build_from_parsed(parsed_js: JsParseOutput): JsBuildOutput;

/**
 * Parse passages only - does not require StoryData
 * Useful for IDE integration where individual files need to be parsed
 */
export function passages(sources_js: JsInputSource[]): Map<string, JsPassage>;

/**
 * Initialize panic hook for better error messages in browser console
 */
export function init_panic_hook(): void;
