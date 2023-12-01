# ZKML for Culinary Categories


Inspired by [this article](https://www.nature.com/articles/srep00196), we trained a deep neural net (DNN) to recognize combinations of ingredients as identifiers for a particular type of cuisine.  
![pair_figure](ingredient_combos.png)

After converting the DNN to a Circom ciruit, we can provide verified inference via Sindri.  Specifically when you make a query, such as "what type of cuisine would use macaroni and parmesan?" and the model returns "Southern European", the response will also include a proof that the answer was achieved by putting your input directly into the model (as opposed to taking any shortcuts).   

This particular model is not large; as the summary below indicates there were only two dense layers and fewer than 25K trainable parameters.  After transpiling, the ZKML circuit has more than 60K constraints.  (You can think of proof time as roughly dependent on the number of constraints.)  The inherent complexity of ML training and inference poses implementation challenges; and now by encapsulating the models within a ZK-container, we are magnifiying that complexity.  It is an [active area](https://github.com/worldcoin/awesome-zkml) of research to accomodate larger models.  Sindri's proof acceleration technique (Sagittal) provides an essential contribution towards the goal of making nontrivial ZKML feasible.
```
Model: "foodie_ml"
_________________________________________________________________
 Layer (type)                Output Shape              Param #   
=================================================================
 input_9 (InputLayer)        [(None, 374)]             0         
                                                                 
 dense_14 (Dense)            (None, 64)                24000     
                                                                 
 re_lu (ReLU)                (None, 64)                0         
                                                                 
 dense_15 (Dense)            (None, 11)                715       
                                                                 
 softmax_8 (Softmax)         (None, 11)                0         
                                                                 
=================================================================
Total params: 24,715
Trainable params: 24,715
Non-trainable params: 0
_________________________________________________________________
```

## Instructions

Clone this repo and set a `SINDRI_API_KEY` environment variable with your API key, e.g. via `export SINDRI_API_KEY=your_key_here`

### 1. Upload

Running the following command will create a circuit and prove it (i.e. the model defined in the `circuit_def/`.)  
```bash
python3 compile_and_prove.py
```
Here is an example printout. It is important that you copy the circuit ID; this will be used as an input each time you query the model.
```
Signing in.
Creating circuit.
   CIRCUIT_ID: 9ef4b718-9f74-4304-9d4a-0a6b864993be
Uploading.
Compiling.
   Circuit poll exited after 230 seconds with status: Ready
```

### 2. Submit an ingredient combination

Now we will enter a list of ingredients to ask the model what type of cuisine this most closely adheres to.  You can get as creative as you want and the model will still return an answer (although you should also try a few obvious ones to get an idea of how accurate the model is, e.g. "ginger garlic soy_sauce" or "tomato olive_oil basil".)
```
python3 query_model.py --circuit "9ef4b718-9f74-4304-9d4a-0a6b864993be" --ingredients "mango soy_sauce peanut_butter spaghetti watermelon beef"
```
Notice from the output of this example that the vocabulary of our model is somewhat limited.  See `vocab.txt` for a full list of all 374 recognizable ingredients.  If you enter a list of ingredients and none are recognized, the script will not bother submitting a Sindri proof request.  
```
Signing in.
Transforming ingredient list to model input.
  spaghetti not found.
Initiating proof.
   Proof poll exited after 13 seconds with status: Ready
   Predicted region: NorthAmerican
Verifying proof.
   Proof was valid
```

## References

Training data came from the supplementary data for this article: https://www.nature.com/articles/srep00196

The conversion from trained a Keras model to a circom circuit was straightforward due to this amazing repo: https://github.com/socathie/keras2circom