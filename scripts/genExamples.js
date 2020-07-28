const { transpile } = require("typescript")
const { readFile, writeFile } = require("fs")
const stringify = require("json-stable-stringify")

process.argv.slice(2).forEach(path => {
    readFile(path, (err, res) => {
        if (err) {
            console.error(err)
            return
        }
        const js = transpile(res.toString(), {})
        function scope(js) {
            var exports = {}
            eval(js)
            return exports
        }
        var module = scope(js)
        var json = stringify(module, { space: 4 })
        var jsonPath = path.replace(/.ts$/, ".json")
        writeFile(jsonPath, json, {}, err => { if (err) console.error(err) })
    })
})
