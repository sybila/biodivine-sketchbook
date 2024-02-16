import { Mode as TextMode } from 'ace-builds/src-noconflict/mode-text'

const TextHighlightRules = ace.require('ace/mode/text_highlight_rules').TextHighlightRules

class AeonHighlightRules extends TextHighlightRules {
  constructor () {
    super()
    this.setKeywords = (kwMap: string) => {
      this.keywordRule.onMatch = this.createKeywordMapper(kwMap, 'identifier')
    }
    this.keywordRule = {
      regex: '\\w+',
      onMatch: function () { return 'text' }
    }
    this.$rules = {
      start: [{
        token: 'keyword.operator',
        regex: '&|\\|'
      }, {
        token: 'paren.lparen',
        regex: '[[({]'
      }, {
        token: 'paren.rparen',
        regex: '[\\])}]'
      },
      this.keywordRule
      ]
    }
    this.normalizeRules()
  }
}

class AeonMode extends TextMode {
  constructor () {
    super()
    this.$id = 'ace/mode/aeon'
    this.$behaviour = this.$defaultBehaviour
    this.HighlightRules = AeonHighlightRules
  }
}

export default AeonMode
