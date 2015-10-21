path = require 'path'

module.exports = (grunt) ->
  grunt.loadNpmTasks 'grunt-bower-task'
  grunt.loadNpmTasks 'grunt-contrib-coffee'
  grunt.loadNpmTasks 'grunt-contrib-copy'
  grunt.loadNpmTasks 'grunt-contrib-less'
  grunt.loadNpmTasks 'grunt-contrib-watch'
  grunt.loadNpmTasks 'grunt-includes'

  grunt.initConfig
    pkg: grunt.file.readJSON 'package.json'

    copy:
      resources:
        files:
          'build/logo.png': '../resources/logo_large.png'
          'build/favicon.png': '../resources/logo_icon.png'

    includes:
      site:
        files: [
          expand: true
          cwd: 'src/'
          src: '*.html',
          dest: 'build/'
        ]

    less:
      site:
        files: [
          expand: true
          cwd: 'src/'
          src: '**/*.less'
          dest: 'build/'
          ext: '.css'
        ]


    coffee:
      site:
        expand: true
        cwd: 'src/'
        src: '**/*.coffee'
        dest: 'build/'
        ext: '.js'

    bower:
      deps:
        options:
          targetDir: 'build/lib/'
          layout: 'byComponent'

    watch:
      html:
        files: 'src/**/*.html'
        tasks: ['includes:site']
      sass:
        files: 'src/**/*.less'
        tasks: ['less:site']
      coffee:
        files: 'src/**/*.coffee'
        tasks: ['coffee:site']

  grunt.registerTask 'default', [
    'includes:site'
    'less:site'
    'coffee:site'
    'copy:resources'
    'bower:deps'
  ]

  null
