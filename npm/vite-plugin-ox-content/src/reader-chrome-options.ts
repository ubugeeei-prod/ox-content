/**
 * Per-control flags for `ssg.readerChrome`.
 *
 * Omitted fields stay on when the feature itself is enabled.
 */
export interface ReaderChromeOptions {
  /**
   * Copy button on fenced code blocks. The clipboard is read in the browser,
   * never at build time.
   *
   * @default true
   */
  copy?: boolean;

  /**
   * Icon and `rel="noopener noreferrer"` on outbound `http(s)` links.
   * Relative, hash, and same-document links are left alone.
   *
   * @default true
   */
  externalLinks?: boolean;

  /**
   * Back-to-top control that appears after the page is scrolled.
   *
   * @default true
   */
  backToTop?: boolean;
}

/**
 * Resolved reader chrome. `false` means no extra markup or JS.
 */
export type ResolvedReaderChrome =
  | false
  | {
      copy: boolean;
      externalLinks: boolean;
      backToTop: boolean;
    };
