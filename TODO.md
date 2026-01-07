# TODO

- [ ] Implement Zigbee/lights tool
- [ ] Create trait for getting tools
- [ ] Implement the some basic openai service to try and use mcp protocol with tools

- [ ] Create awakening loop and breakout word

**these might be the same service**

- [ ] Implement whisper service for reading voice
- [ ] Implement brain, openai service to send tools and get some json back to call my tools
- [ ] Implement talker service to use audio

- [ ] Implement service for sending out messsages to telegram

**tools**

- [ ] Weather tool
- [ ] transport tool (ruter)
- [ ] Talking tool

**misc**

- [ ]

**setup**

- [ ] Create startup script to spawn backend, mosquito and zigbee2mqtt
- [ ] Create install script to be used on the rasberry
- [ ] Install on rasberry, attach microfone,

### How

- awakening loop
  - Free voice reader program (since its on forever)
  - listens and only activates if it hears the activation word
- inner loop
  - listens to input
  - if hears close down word, it breaks out of the loop and sleeps
  - if not, sends the prompt to the brain, the brain uses system prompts and talks to openai
  - open ai returns some data object to be used on one of the tools
  - tool is executed and the result is read back
  - inner loop goes to top and listens for new input
