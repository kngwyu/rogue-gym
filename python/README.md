# rogue-gym python API

You can use rogue-gym as [Open AI Gym](https://github.com/openai/gym) environment.

# API documents
COMING SOON

# Example

```bash
>>> from rogue_gym.envs import RogueEnv
>>> CONFIG = {
...     'width': 32, 'height': 16,
...     'dungeon': {
...         'style': 'rogue',
...         'room_num_x': 2, 'room_num_y': 2
...      }
... }
>>> env = RogueEnv(max_steps=100, config_dict=CONFIG)
>>> rewards = 0
>>> state = env.reset()
>>> for i in range(10): 
...     # move right
...     state, reward, done, _ = env.step('l')
...     rewards += reward
... 
>>> env
                                
                                
                       ---      
                       .@|      
                       ..|      
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
                                
Level:  1 Gold:     0 Hp: 12(12) Str: 16(16) Arm:  0 Exp:  1/ 0 
```


# Developper notes
Build wheel 
```
cd ../
docker run --rm -v $PWD:/io quay.io/pypa/manylinux1_x86_64 /io/build-wheels.sh
```
