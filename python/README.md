# rogue-gym python API

You can use rogue-gym as [Open AI Gym](https://github.com/openai/gym) environment.

# API documents
COMING SOON

# Example

```python
>>> from rogue_gym.envs import RogueEnv
>>> env = RogueEnv(seed=1, config_dict={"width":36,"height":18})
>>> env
                                    
                                    
                                    
                                    
                                    
                                    
                                    
                                    
                                    
                                    
                                    
                                    
                        ---------   
                        |..*....|   
                        +......@|   
                        ---------   
                                    
                                    
Level:  1 Gold:     0 Hp: 12(12) Str: 16(16) Arm:  0 Exp:  1/ 0
```


# Developper notes
Build wheel 
```
cd rogue_gym
docker run --rm -v $PWD:/io quay.io/pypa/manylinux1_x86_64 /io/build-wheels.sh
```
