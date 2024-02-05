const oop = ace.require('ace/lib/oop')
const TextMode = ace.require('ace/mode/text').Mode
const TextHighlightRules = ace.require('ace/mode/text_highlight_rules').TextHighlightRules
const AeonHighlightRules = function () {
  this.setKeywords = function (kwMap) {
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

oop.inherits(AeonHighlightRules, TextHighlightRules)
const AeonMode = function () {
  this.HighlightRules = AeonHighlightRules
}
oop.inherits(AeonMode, TextMode);

(function () {
  this.$id = 'ace/mode/aeon'
  this.$behaviour = this.$defaultBehaviour
}).call(AeonMode.prototype)

export default AeonMode
