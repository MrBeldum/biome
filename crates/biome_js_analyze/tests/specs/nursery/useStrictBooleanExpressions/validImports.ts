// should not generate diagnostics
import * as ns from "./exports";
const { optionalFlag = false } = ns;
if (optionalFlag) {}
