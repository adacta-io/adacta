from require import *



@require(app='adacta.backend.web:App')
def main(app):
    app.run()



if __name__ == '__main__':
    main()
