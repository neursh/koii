import { hookstate } from 'hookstate';

export default abstract class GlobalContext {
  /**
   * Current location of the parent based on the displaying parent, NOT based on the current URL route. Example: "/", "/works",...
   */
  static currentParent = hookstate('');

  /**
   * When the page is rendered, it will turns this to true,
   * any parent page navigation will causes this to goes false.
   */
  static pageRendered = hookstate(false);

  /**
   * This value is used in a primitive base, it should only be set as/before the `Pipeline` component renders.
   *
   * Notify `LoadingFallback` to show the background to hide stuff or not.
   * The value will be set back to `false` after `LoadingFallback` read the value for the transition.
   *
   * **TO AVOID FLICKERS. BEST IS TO NOTIFY AT THE SAME LEVEL AS `Pipeline`!!! IDEALLY IN `index.tsx`!!!!**
   *
   * Every time you invoke this variable, remember to comment what are you doing with it.
   */
  static loaderDoNotHide = hookstate(false);
}
